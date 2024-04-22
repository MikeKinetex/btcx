import { HardhatRuntimeEnvironment } from 'hardhat/types';
import { beginTask } from '../../../utils/format';
import { operation } from '../../../utils/operation';
import { attachContract } from '../../../utils/attach';
import { SuccinctSubmitter } from '../../../../typechain-types';

type Args = {
  target: string;
  parentHash: string;
  nHeaders: number;
  dry: boolean;
  nonce?: string;
};

export const task = async (
  args: Args,
  env: HardhatRuntimeEnvironment,
): Promise<void> => {
  beginTask();

  await operation({
    title: 'Request update headers from SuccinctSubmitter',
    env,
    mode: args.dry ? 'dry-run' : 'run',
    transaction: async () => {
      const succinctSubmitter = await attachContract<SuccinctSubmitter>({
        contractName: 'SuccinctSubmitter',
        contractAddress: args.target,
        env,
      });
      const data = succinctSubmitter.interface.encodeFunctionData(
        'request',
        [
          args.parentHash,
          args.nHeaders,
        ],
      );
      return { data, to: args.target, value: 300000 };
    },
    nonce: args.nonce,
  });
};
