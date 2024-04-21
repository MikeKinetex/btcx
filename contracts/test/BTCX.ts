import { ethers } from 'hardhat';
import { loadFixture } from '@nomicfoundation/hardhat-network-helpers';
import { getContractAddress } from '@ethersproject/address';
import { expect } from 'chai';

type Header = {
  height: number;
  hash: string;
  header: string;
}

const reverseEndianness = (s: string) => {
  const result = [];
  let len = s.length - 2;
  while (len >= 0) {
    result.push(s.substr(len, 2));
    len -= 2;
  }
  return result.join('');
}

describe('BTCX', function () {
  const fixturePath = `${__dirname}/fixtures`;

  const loadHeaders = async (start: number, end: number) => (await import(`${fixturePath}/headers-${start}-${end}.json`)).default;

  async function deployFixture(genesisBlockHeight: number, genesisBlockHash: string, genesisTarget: string) {
    const [owner] = await ethers.getSigners();

    const nonce = await owner.getNonce();
    const btcxAddress = getContractAddress({
      from: owner.address,
      nonce: nonce + 2
    });

    const EVMVerifier = await ethers.getContractFactory('EVMVerifier');
    const evmVerifier = await EVMVerifier.connect(owner).deploy();

    const EVMSubmitter = await ethers.getContractFactory('EVMSubmitter');
    const evmSubmitter = await EVMSubmitter.deploy(btcxAddress, evmVerifier);

    const BTCX = await ethers.getContractFactory('BTCX');
    const btcx = await BTCX.deploy(
      genesisBlockHeight,
      '0x' + reverseEndianness(genesisBlockHash),
      ethers.ZeroHash,
      genesisTarget,
      [ evmSubmitter ]
    );

    return {
      btcx,
      evmSubmitter,
      evmVerifier
    };
  }

  async function deployFixture_genesis() {
    const blockHeaders = await loadHeaders(0, 2015);
    return deployFixture(0, blockHeaders[0].hash, '0xFFFF0000000000000000000000000000000000000000000000000000');
  }

  async function deployFixture_201600() {
    const blockHeaders = await loadHeaders(201600, 211600);
    return deployFixture(201600, blockHeaders[0].hash, '0x057e080000000000000000000000000000000000000000000000');
  }

  it('Should submit block headers without retargeting (no fork)', async function () {
    const { btcx, evmSubmitter } = await loadFixture(deployFixture_genesis);
    const blockHeaders = await loadHeaders(0, 2015);

    await evmSubmitter.submit(
      '0x' + reverseEndianness(blockHeaders[0].hash),
      '0x' + blockHeaders.slice(1, 501).map((h:Header) => h.header).join(''),
    );

    expect(await btcx.bestBlockHeight()).to.be.equal(500);
    expect((await btcx.bestBlockHash()).slice(2)).to.be.equal(
      reverseEndianness(blockHeaders[500].hash)
    );
  });

  it('Should submit block headers without retargeting (with fork)', async function () {
    const { btcx, evmSubmitter } = await loadFixture(deployFixture_genesis);
    const blockHeaders = await loadHeaders(0, 2015);

    await evmSubmitter.submit(
      '0x' + reverseEndianness(blockHeaders[0].hash),
      '0x' + blockHeaders.slice(1, 451).map((h:Header) => h.header).join(''),
    );

    await evmSubmitter.submit(
      '0x' + reverseEndianness(blockHeaders[400].hash),
      '0x' + blockHeaders.slice(401, 501).map((h:Header) => h.header).join(''),
    );

    expect(await btcx.bestBlockHeight()).to.be.equal(500);
    expect((await btcx.bestBlockHash()).slice(2)).to.be.equal(
      reverseEndianness(blockHeaders[500].hash)
    );
  });

  it('Should submit block headers without retargeting (fork with less work)', async function () {
    const { btcx, evmSubmitter } = await loadFixture(deployFixture_genesis);
    const blockHeaders = await loadHeaders(0, 2015);

    await evmSubmitter.submit(
      '0x' + reverseEndianness(blockHeaders[0].hash),
      '0x' + blockHeaders.slice(1, 501).map((h:Header) => h.header).join(''),
    );

    expect(evmSubmitter.submit(
      '0x' + reverseEndianness(blockHeaders[400].hash),
      '0x' + blockHeaders.slice(401, 451).map((h:Header) => h.header).join(''),
    )).to.be.revertedWithCustomError(btcx, 'ForksNotSupported()');

    expect(await btcx.bestBlockHeight()).to.be.equal(500);
    expect((await btcx.bestBlockHash()).slice(2)).to.be.equal(
      reverseEndianness(blockHeaders[500].hash)
    );
  });

  it('Should submit block headers with retargeting (no fork)', async function () {
    const { btcx, evmSubmitter } = await loadFixture(deployFixture_201600);
    const blockHeaders = await loadHeaders(201600, 211600);

    for (let i = 0; i < 5; i++) {
      await evmSubmitter.submit(
        '0x' + reverseEndianness(blockHeaders[i * 500].hash),
        '0x' + (i == 4 ? (blockHeaders[0].header + blockHeaders[2015].header) : '') + blockHeaders.slice((i * 500) + 1, 1 + ((i + 1) * 500)).map((h:Header) => h.header).join('')
      );
    }

    expect(await btcx.bestBlockHeight()).to.be.equal(201600 + 2500);
    expect((await btcx.bestBlockHash()).slice(2)).to.be.equal(
      reverseEndianness(blockHeaders[2500].hash)
    );
  });

  it('Should submit block headers with retargeting (with fork)', async function () {
    const { btcx, evmSubmitter } = await loadFixture(deployFixture_201600);
    const blockHeaders = await loadHeaders(201600, 211600);

    for (let i = 0; i < 5; i++) {
      await evmSubmitter.submit(
        '0x' + reverseEndianness(blockHeaders[i * 500].hash),
        '0x' + (i == 4 ? (blockHeaders[0].header + blockHeaders[2015].header) : '') + blockHeaders.slice((i * 500) + 1, 1 + ((i + 1) * 500 - (i == 4 ? 100 : 0))).map((h:Header) => h.header).join('')
      );
    }

    await evmSubmitter.submit(
      '0x' + reverseEndianness(blockHeaders[2000].hash),
      '0x' + (blockHeaders[0].header + blockHeaders[2015].header) + blockHeaders.slice(2001, 2501).map((h:Header) => h.header).join('')
    );

    expect(await btcx.bestBlockHeight()).to.be.equal(201600 + 2500);
    expect((await btcx.bestBlockHash()).slice(2)).to.be.equal(
      reverseEndianness(blockHeaders[2500].hash)
    );
  });

  it('Should submit block headers with retargeting (fork with less work)', async function () {
    const { btcx, evmSubmitter } = await loadFixture(deployFixture_201600);
    const blockHeaders = await loadHeaders(201600, 211600);

    for (let i = 0; i < 5; i++) {
      await evmSubmitter.submit(
        '0x' + reverseEndianness(blockHeaders[i * 500].hash),
        '0x' + (i == 4 ? (blockHeaders[0].header + blockHeaders[2015].header) : '') + blockHeaders.slice((i * 500) + 1, 1 + ((i + 1) * 500)).map((h:Header) => h.header).join('')
      );
    }

    expect(evmSubmitter.submit(
      '0x' + reverseEndianness(blockHeaders[2000].hash),
      '0x' + (blockHeaders[0].header + blockHeaders[2015].header) + blockHeaders.slice(2001, 2451).map((h:Header) => h.header).join('')
    )).to.be.revertedWithCustomError(btcx, 'ForksNotSupported()');

    expect(await btcx.bestBlockHeight()).to.be.equal(201600 + 2500);
    expect((await btcx.bestBlockHash()).slice(2)).to.be.equal(
      reverseEndianness(blockHeaders[2500].hash)
    );
  });

});