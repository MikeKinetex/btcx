import { task } from 'hardhat/config';

// Address

task('address-predict', 'Predicts address of deployed contract')
  .addOptionalParam('creator', 'Contract creator account (defaults to deployer)')
  .addOptionalParam('nonce', 'Transaction nonce (defaults to next transaction)')
  .setAction(async (args, env) => {
    const { task } = await import('./address/predict');
    await task(args, env);
  });

// Deployer

task('deployer', 'Prints active deployer account info')
  .setAction(async (args, env) => {
    const { task } = await import('./deployer/info');
    await task(args, env);
  });

// Contract

task('contract-selector-list', 'Lists contract function selectors')
  .addParam('contract', 'Contract name to list function selectors of')
  .setAction(async (args, env) => {
    const { task } = await import('./contract/selector/list');
    await task(args, env);
  });

task('contract-call-encode', 'Encodes contract function call data')
  .addParam('contract', 'Contract name to encode function call for')
  .addParam('function', 'Function name, signature, or selector to encode')
  .addOptionalParam('params', 'Function call parameters to encode (JSON)')
  .setAction(async (args, env) => {
    const { task } = await import('./contract/call/encode');
    await task(args, env);
  });

// BTCX

task('deploy-btcx', 'Deploys BTCX')
  .addParam('publisher')
  .addFlag('dry', 'Perform a dry run (estimate only)')
  .addOptionalParam('nonce', 'Nonce override')
  .setAction(async (args, env) => {
    const { task } = await import('./btcx/deploy');
    await task(args, env);
  });

// Submitters

task('deploy-succinct-submitter', 'Deploys SuccinctSubmitter')
  .addParam('lightClient')
  .addParam('gateway')
  .addParam('verifyFunctions')
  .addParam('retargetFunctions')
  .addFlag('dry', 'Perform a dry run (estimate only)')
  .addOptionalParam('nonce', 'Nonce override')
  .setAction(async (args, env) => {
    const { task } = await import('./submitters/succinct/deploy');
    await task(args, env);
  });

task('deploy-evm-submitter', 'Deploys EVMSubmitter')
  .addParam('lightClient')
  .addParam('blockVerifier')
  .addFlag('dry', 'Perform a dry run (estimate only)')
  .addOptionalParam('nonce', 'Nonce override')
  .setAction(async (args, env) => {
    const { task } = await import('./submitters/evm/deploy');
    await task(args, env);
  });

task('deploy-evm-verifier', 'Deploys EVMVerifier')
  .addFlag('dry', 'Perform a dry run (estimate only)')
  .addOptionalParam('nonce', 'Nonce override')
  .setAction(async (args, env) => {
    const { task } = await import('./submitters/evm/verifier/deploy');
    await task(args, env);
  });