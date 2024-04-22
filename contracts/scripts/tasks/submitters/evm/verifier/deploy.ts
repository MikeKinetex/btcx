import { HardhatRuntimeEnvironment } from 'hardhat/types';
import { getDeployContractData } from '../../../../utils/deploy';
import { beginTask } from '../../../../utils/format';
import { operation } from '../../../../utils/operation';

type Args = {
  dry: boolean;
  nonce?: string;
};

export const task = async (
  args: Args,
  env: HardhatRuntimeEnvironment,
): Promise<void> => {
  beginTask();

  await operation({
    title: 'Deploy EVMVerifier',
    env,
    mode: args.dry ? 'dry-run' : 'run',
    transaction: async () => {
      const data = await getDeployContractData({
        contractName: 'EVMVerifier',
        constructorParams: [],
        env,
      });
      return { data };
    },
    nonce: args.nonce,
  });
};
