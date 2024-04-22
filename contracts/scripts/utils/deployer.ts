import { SignerWithAddress } from '@nomicfoundation/hardhat-ethers/signers';
import { HardhatRuntimeEnvironment } from 'hardhat/types';

export const getDeployer = async (env: HardhatRuntimeEnvironment): Promise<SignerWithAddress> => {
  const [deployer] = await env.ethers.getSigners();
  return deployer;
};
