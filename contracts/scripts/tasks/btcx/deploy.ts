import { HardhatRuntimeEnvironment } from 'hardhat/types';
import { getDeployContractData } from '../../utils/deploy';
import { beginTask } from '../../utils/format';
import { operation } from '../../utils/operation';

type Args = {
  genesisBlockHeight: number;
  genesisBlockHash: string;
  genesisBlockUtreexo: string;
  genesisBlockTarget: string,
  submitters: string,
  dry: boolean;
  nonce?: string;
};

export const task = async (
  args: Args,
  env: HardhatRuntimeEnvironment,
): Promise<void> => {
  beginTask();

  await operation({
    title: 'Deploy BTCX',
    env,
    mode: args.dry ? 'dry-run' : 'run',
    transaction: async () => {
      const data = await getDeployContractData({
        contractName: 'BTCX',
        constructorParams: [
          args.genesisBlockHeight,
          args.genesisBlockHash,
          args.genesisBlockUtreexo,
          args.genesisBlockTarget,
          args.submitters.split(',')
        ],
        env,
      });
      return { data };
    },
    nonce: args.nonce,
  });
};
