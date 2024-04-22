import { BytesLike } from 'ethers';
import { SignerWithAddress } from '@nomicfoundation/hardhat-ethers/signers';

const ETH_DECIMALS = 18;
const GWEI_DECIMALS = 9;

const toDecimalParts = (wei: bigint, baseDecimals: number, toDecimals: number): [bigint, bigint, number] => {
  const integerDenominator = 10n ** BigInt(baseDecimals);
  const integer = wei / integerDenominator;

  const fractionDenominator = 10n ** BigInt(baseDecimals - toDecimals);
  const fractionSubtrahend = (10n ** BigInt(toDecimals)) * integer;
  const fraction = (wei / fractionDenominator) - fractionSubtrahend;
  const fractionZeros = toDecimals - fraction.toString().length;

  return [integer, fraction, fractionZeros];
};

const absBigInt = (value: bigint): bigint => {
  return value < 0 ? -value : value;
}

const toDecimal = (wei: bigint, baseDecimals: number, toDecimals: number): string => {
  const [integer, fraction, fractionZeros] = toDecimalParts(absBigInt(wei), baseDecimals, toDecimals);
  const fractionPrefix = '0'.repeat(fractionZeros);
  const decimalStr = `${wei < 0n ? '-' : ''}${integer.toString()}.${fractionPrefix}${fraction.toString()}`;
  return decimalStr;
};

export const formatEthString = (wei: bigint, decimals: number): string => {
  const decimalStr = toDecimal(wei, ETH_DECIMALS, decimals);
  const str = `${decimalStr} ETH`;
  return str;
};

const formatWeiString = (wei: bigint): string => {
  const str = `${wei.toString()} WEI`;
  return str;
};

const formatGweiString = (wei: bigint, decimals: number): string => {
  const decimalStr = toDecimal(wei, GWEI_DECIMALS, decimals);
  const str = `${decimalStr} GWEI`;
  return str;
};

export const formatBalanceSummary = (balance: bigint): string => {
  const summary = `~${formatEthString(balance, 6)} (${formatWeiString(balance)})`;
  return summary;
};

export const getBalanceSummary = async (address: SignerWithAddress): Promise<string> => {
  const balance = await address.provider.getBalance(address);
  const summary = formatBalanceSummary(balance);
  return summary;
};

interface FormatBalanceDeltaSummaryOptions {
  abs?: boolean;
}

export const formatBalanceDeltaSummary = (
  balanceBefore: bigint,
  balanceAfter: bigint,
  options: FormatBalanceDeltaSummaryOptions = {},
): string => {
  const { abs = false } = options;

  const balanceDelta = balanceAfter - balanceBefore;
  let summary = formatBalanceSummary(absBigInt(balanceDelta));
  if (abs) {
    return summary;
  }

  if (balanceDelta > 0n) {
    summary = `+ ${summary}`;
  } else if (balanceDelta < 0n) {
    summary = `- ${summary}`;
  }
  return summary;
};

export const formatGasSummary = (gasPrice: bigint): string => {
  const summary = `~${formatGweiString(gasPrice, 6)} (${formatWeiString(gasPrice)})`;
  return summary;
};

export const formatGasUnitsSummary = (gasUnits: bigint): string => {
  const summary = gasUnits.toString();
  return summary;
};

export const getHexBytesSize = (data?: BytesLike| null): number => {
  let bytes: number;
  if (data == null) {
    // Nothing
    bytes = 0;
  } else if (typeof data === 'string') {
    // string string
    let hexLength = data.length;
    if (data.startsWith('0x')) {
      hexLength -= 2;
    }
    bytes = Math.ceil(hexLength / 2);
  } else {
    // Byte array
    bytes = data.length;
  }
  return bytes;
}

export const makeHexBytesSize = (bytes: number): string => {
  return `${bytes} bytes`;
};

export const formatHexBytesSize = (data?: BytesLike | null): string => {
  const bytes = getHexBytesSize(data);
  const str = makeHexBytesSize(bytes);
  return str;
};

export const formatBoolean = (value: boolean): string => {
  return value ? 'true' : 'false';
};

/**
 * Every block printed should leave spacing after itself.
 * This function adds spacing before any other output
 */
export const beginTask = (): void => {
  console.log();
};

type Optional<T> = T | null | undefined;

export const formatOptional = <T>(value: Optional<T>, formatter?: (value: T) => string): string => {
  if (value == null) {
    return '<nullish>';
  }
  if (formatter == null) {
    return `${value}`;
  }
  return formatter(value);
}
