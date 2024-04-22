import { ethers } from 'hardhat';
import { HardhatRuntimeEnvironment } from 'hardhat/types';
import { beginTask } from '../../../utils/format';
import { operation } from '../../../utils/operation';

type Args = {
  contract: string;
};

export const task = async (
  args: Args,
  env: HardhatRuntimeEnvironment,
): Promise<void> => {
  beginTask();

  await operation({
    title: 'List contract selectors',
    env,
    mode: 'read',
    transaction: async () => {
      const contractFactory = await ethers.getContractFactory(args.contract);

      const result: unknown[] = [];
      contractFactory.interface.forEachFunction((func) => {
        result.push({ selector: func.selector, signature: func.format() });
      });

      console.table(result);
      console.log();

      return undefined;
    },
  });
};
