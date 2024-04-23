import { TransactionLike, TransactionRequest } from 'ethers';
import { HardhatRuntimeEnvironment } from 'hardhat/types';
import { getDeployer } from './deployer';
import {
  formatBalanceDeltaSummary,
  formatBalanceSummary,
  formatGasSummary,
  formatGasUnitsSummary,
  formatHexBytesSize,
  formatOptional,
} from './format';
import { logPropertyGroup, Property } from './property';
import { required } from './required';

const EIP1559_INCOMPATIBLE_CHAINS = new Set<string>([]);

export type Transaction = {
  data?: string; // ('run'-like mode only)
  to?: string; // No value => deploy contract ('run'-like mode only)
  value?: string; // ('run'-like mode only)
  result?: unknown; // ('read' mode only)
}

export type OperationMode = 'run' | 'dry-run' | 'read';

export type Operation = {
  title: string;
  env: HardhatRuntimeEnvironment;
  mode: OperationMode;
  transaction: () => Promise<Transaction | undefined>;
  nonce?: string;
  gasPrice?: string;
}

export const operation = async (op: Operation): Promise<void> => {
  const deployer = await getDeployer(op.env);
  const txCount = await deployer.provider.getTransactionCount(deployer);
  const balanceBefore = await deployer.provider.getBalance(deployer);
  const feeData = await deployer.provider.getFeeData();
  const gasPrice = required('gas price', feeData.gasPrice);
  const maxGasPrice = feeData.maxFeePerGas;

  logPropertyGroup({
    title: `Operation "${op.title}" started`,
    properties: [
      { title: 'network', value: op.env.network.name },
      { title: 'mode', value: op.mode },
      { title: 'deployer', value: deployer.address },
      { title: 'tx count', value: txCount.toString() },
      { title: 'balance', value: formatBalanceSummary(balanceBefore) },
      {
        title: 'fee data',
        value: [
          { title: 'max priority fee per gas', value: formatOptional(feeData.maxPriorityFeePerGas, formatGasSummary) },
          { title: 'gas price', value: formatOptional(feeData.gasPrice, formatGasSummary) },
          { title: 'max fee per gas', value: formatOptional(feeData.maxFeePerGas, formatGasSummary) },
        ],
      },
    ],
  });

  const transaction = await op.transaction();

  if (!transaction) {
    logPropertyGroup({
      title: `Operation "${op.title}" finished (no tx data)`,
      properties: [],
    });
    return;
  }

  let transactionRequest: TransactionLike<string> | undefined;
  if (op.mode !== 'read') {
    const shouldUseLegacyType = (): boolean => {
      const isDeploy = transaction.to == null && transaction.data != null;
      if (isDeploy) {
        return true;
      }

      const isIncompatible = EIP1559_INCOMPATIBLE_CHAINS.has(op.env.network.name);
      if (isIncompatible) {
        return true;
      }

      return false;
    };

    const tx: TransactionRequest = {
      from: deployer.address,
      to: transaction.to,
      data: transaction.data,
      value: transaction.value,
      type: shouldUseLegacyType() ? 0 : 2,
    };
    transactionRequest = await deployer.populateTransaction(tx);

    const overrideProps: Property[] = [];
    if (op.nonce != null) {
      overrideProps.push({ title: 'nonce override', value: op.nonce });
      transactionRequest.nonce = Number(op.nonce);
    }

    if (op.gasPrice != null) {
      overrideProps.push({ title: 'gas price override', value: op.gasPrice });
      transactionRequest.gasPrice = op.gasPrice;
    }

    if (overrideProps.length > 0) {
      logPropertyGroup({
        title: `Operation "${op.title}" overrides`,
        properties: overrideProps,
      });
    }
  }

  const extraProps: Property[] = [];

  let resultProp: Property | undefined;
  if (transaction.result != null) {
    resultProp = { title: 'result', value: JSON.stringify(transaction.result) };
    extraProps.push(resultProp);
  }

  let gasUnits = 0n;
  if (op.mode === 'read') {
    logPropertyGroup({
      title: `Operation "${op.title}" read`,
      properties: resultProp != null ? [resultProp] : [],
    });
  } else {
    transactionRequest = required('transaction request', transactionRequest);
    gasUnits = await deployer.estimateGas(transactionRequest);

    const estimatedGasCost = gasUnits * gasPrice;
    const estimatedMaxGasCost = gasUnits * (maxGasPrice ?? gasPrice);
    const estimatedBalanceAfter = balanceBefore - estimatedGasCost;
    const estimatedBalanceAfterMax = balanceBefore - estimatedMaxGasCost;

    logPropertyGroup({
      title: `Operation "${op.title}" estimate`,
      properties: [
        { title: 'data size', value: formatHexBytesSize(transaction.data) },
        { title: 'gas units', value: formatGasUnitsSummary(gasUnits) },
        {
          title: 'gas spend',
          value: [
            { title: 'balance before', value: formatBalanceSummary(balanceBefore) },
            { title: 'balance after', value: formatBalanceSummary(estimatedBalanceAfter) },
            { title: 'operation cost', value: formatBalanceDeltaSummary(balanceBefore, estimatedBalanceAfter, { abs: true }) },
          ],
        },
        {
          title: 'max gas spend',
          value: [
            { title: 'balance before', value: formatBalanceSummary(balanceBefore) },
            { title: 'balance after', value: formatBalanceSummary(estimatedBalanceAfterMax) },
            { title: 'operation cost', value: formatBalanceDeltaSummary(balanceBefore, estimatedBalanceAfterMax, { abs: true }) },
          ],
        },
      ],
    });
  }

  if (op.mode === 'run') {
    transactionRequest = required('transaction request', transactionRequest);
    const transactionResponse = await deployer.sendTransaction(transactionRequest);
    const txidProp = { title: 'txid', value: transactionResponse.hash };
    extraProps.push(txidProp);

    logPropertyGroup({
      title: `Operation "${op.title}" waiting for transaction to finish`,
      properties: [
        txidProp,
      ],
    });

    const transactionReceipt = await transactionResponse.wait();
    if (transactionReceipt == null) {
      throw new Error('Empty transaction receipt');
    }

    const contractAddressProp = { title: 'contract address', value: formatOptional(transactionReceipt.contractAddress) };
    extraProps.push(contractAddressProp);
    const blockProp = { title: 'block', value: transactionReceipt.blockNumber.toString() };
    extraProps.push(blockProp);

    logPropertyGroup({
      title: `Operation "${op.title}" transaction finished`,
      properties: [
        txidProp,
        blockProp,
        contractAddressProp,
      ],
    });
  }

  const balanceAfter = op.mode === 'run' ? await deployer.provider.getBalance(deployer) : balanceBefore;

  logPropertyGroup({
    title: `Operation "${op.title}" finished`,
    properties: [
      { title: 'network', value: op.env.network.name },
      { title: 'mode', value: op.mode },
      { title: 'deployer', value: deployer.address },
      { title: 'balance before', value: formatBalanceSummary(balanceBefore) },
      { title: 'balance after', value: formatBalanceSummary(balanceAfter) },
      { title: 'operation cost', value: formatBalanceDeltaSummary(balanceBefore, balanceAfter, { abs: true }) },
      { title: 'gas units', value: formatGasUnitsSummary(gasUnits) },
      ...extraProps,
    ],
  });
}
