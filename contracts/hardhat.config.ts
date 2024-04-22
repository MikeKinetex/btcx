import { HardhatUserConfig } from 'hardhat/config';
import '@nomicfoundation/hardhat-toolbox';
import '@nomiclabs/hardhat-solhint';
import 'hardhat-contract-sizer';
import 'dotenv/config';
import './scripts/tasks';

const MAINNET_RPC_URL = process.env.MAINNET_RPC_URL || '';
const SEPOLIA_RPC_URL = process.env.SEPOLIA_RPC_URL || '';

const MAINNET_EXPLORER_API_KEY = process.env.MAINNET_EXPLORER_API_KEY || '';

const EMPTY_PRIVATE_KEY = '0000000000000000000000000000000000000000000000000000000000000000';
const DEPLOYER_PRIVATE_KEY = process.env.DEPLOYER_PRIVATE_KEY || EMPTY_PRIVATE_KEY;

const config: HardhatUserConfig = {
  solidity: {
    version: '0.8.24',
    settings: {
      optimizer: {
        enabled: true,
        runs: 1_000_000,
      },
    },
  },
  networks: {
    mainnet: {
      url: MAINNET_RPC_URL,
      accounts: [DEPLOYER_PRIVATE_KEY],
    },
    sepolia: {
      url: SEPOLIA_RPC_URL,
      accounts: [DEPLOYER_PRIVATE_KEY],
    },
  },
  etherscan: {
    apiKey: {
      mainnet: MAINNET_EXPLORER_API_KEY,
    },
  },
  gasReporter: {
    enabled: true,
    currency: 'USD',
  },
  contractSizer: {
    alphaSort: false,
    disambiguatePaths: true,
    runOnCompile: true,
    strict: true,
    only: [],
  },
  paths: {
    sources: "./src",
  },
};

export default config;