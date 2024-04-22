import { HardhatRuntimeEnvironment } from 'hardhat/types';
import { getDeployContractData } from '../../../utils/deploy';
import { beginTask } from '../../../utils/format';
import { operation } from '../../../utils/operation';

type Args = {
  lightClient: string;
  gateway: string;
  verifyFunctions: string;
  retargetFunctions: string;
  dry: boolean;
  nonce?: string;
};

export const task = async (
  args: Args,
  env: HardhatRuntimeEnvironment,
): Promise<void> => {
  beginTask();

  await operation({
    title: 'Deploy SuccinctSubmitter',
    env,
    mode: args.dry ? 'dry-run' : 'run',
    transaction: async () => {
      const data = await getDeployContractData({
        contractName: 'SuccinctSubmitter',
        constructorParams: [
          args.lightClient,
          args.gateway,
          args.verifyFunctions.split(',').map((f) => { const fc = f.split(':'); return { functionId: fc[0], nHeaders: fc[1] } }),
          args.retargetFunctions.split(',').map((f) => { const fc = f.split(':'); return { functionId: fc[0], nHeaders: fc[1] } }),
        ],
        env,
      });
      return { data };
    },
    nonce: args.nonce,
  });
};
