import { getCreateAddress } from 'ethers';
import { HardhatRuntimeEnvironment } from 'hardhat/types';
import { beginTask } from '../../utils/format';
import { getDeployer } from '../../utils/deployer';
import { logPropertyGroup } from '../../utils/property';

type Args = {
  creator?: string; // Defaults to active deployer
  nonce?: string; // Defaults to nonce of creator's next tx
};

export const task = async (
  args: Args,
  env: HardhatRuntimeEnvironment,
): Promise<void> => {
  beginTask();

  const getCreator = async (): Promise<string> => {
    if (args.creator != null) {
      return args.creator;
    }

    const deployer = await getDeployer(env);
    const deployerAddress = await deployer.getAddress();
    return deployerAddress;
  };

  const getNonce = async (creator: string): Promise<string> => {
    if (args.nonce != null) {
      return args.nonce;
    }

    const nextTxNonce = await env.ethers.provider.getTransactionCount(creator);
    return nextTxNonce.toString();
  };

  const predictAddress = (creator: string, nonce: string): string => {
    const deployAddress = getCreateAddress({ from: creator, nonce });
    return deployAddress;
  };

  const creator = await getCreator();
  const nonce = await getNonce(creator);
  const address = predictAddress(creator, nonce);

  logPropertyGroup({
    title: `Deploy address prediction`,
    properties: [
      { title: 'creator', value: creator },
      { title: 'nonce', value: nonce },
      { title: 'address', value: address },
    ],
  });
};
