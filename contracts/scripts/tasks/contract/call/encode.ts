import { ethers } from 'hardhat';
import { HardhatRuntimeEnvironment } from 'hardhat/types';
import { beginTask } from '../../../utils/format';
import { operation } from '../../../utils/operation';
import { FunctionFragment } from 'ethers';

type Args = {
  contract: string;
  function: string;
  params?: string;
};

export const task = async (
  args: Args,
  env: HardhatRuntimeEnvironment,
): Promise<void> => {
  beginTask();

  await operation({
    title: 'Encode contract function call',
    env,
    mode: 'read',
    transaction: async () => {
      const contractFactory = await ethers.getContractFactory(args.contract);

      const targetFunctions: FunctionFragment[] = [];
      contractFactory.interface.forEachFunction((func) => {
        if (func.selector === args.function || func.name === args.function || func.format() === args.function) {
          targetFunctions.push(func);
        }
      });

      if (targetFunctions.length !== 1) {
        throw new Error(
          `Found ${targetFunctions.length} functions matching "${args.function}" ` +
          `in "${args.contract}" contract, exactly 1 expected`
        );
      }

      const targetFunction = targetFunctions[0];
      const functionParams = args.params ? JSON.parse(args.params) : undefined;
      const functionData = contractFactory.interface.encodeFunctionData(targetFunction, functionParams);

      return {
        result: functionData,
      };
    },
  });
};
