import { ethers } from 'hardhat';
import { loadFixture } from '@nomicfoundation/hardhat-network-helpers';
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

describe('EVMVerifier', function () {
  const fixturePath = `${__dirname}/fixtures`;

  const loadHeaders = async (start: number, end: number) => (await import(`${fixturePath}/headers-${start}-${end}.json`)).default;
  const deployFixture = async () => (await ethers.getContractFactory('EVMVerifier')).deploy();

  it('Should verify block header sequence without retargeting', async function () {
    const evmVerifier = await loadFixture(deployFixture);
    const blockHeaders = await loadHeaders(0, 2015);

    const endHash = await evmVerifier.verify(
      0,
      ethers.ZeroHash,
      '0x00000000FFFF0000000000000000000000000000000000000000000000000000',
      '0x' + blockHeaders.slice(0, 2015).map((h:Header) => h.header).join(''),
    );

    expect(endHash.slice(2)).to.be.equal(
      reverseEndianness(blockHeaders[blockHeaders.length - 1].hash)
    );
  });

  it('Should verify block header sequence with retargeting', async function () {
    const evmVerifier = await loadFixture(deployFixture);
    const blockHeaders:Array<Header> = await loadHeaders(201600, 211600);

    const result = await evmVerifier.verifyWithRetargeting(
      201600, // parent block height
      '0x' + reverseEndianness(blockHeaders[0].hash), // parent block hash
      '0x' + reverseEndianness(blockHeaders[0].hash), // start period hash
      '0x000000000000057e080000000000000000000000000000000000000000000000',
      '0x' + blockHeaders[0].header + blockHeaders[2015].header + blockHeaders.slice(1, 6000).map((h:Header) => h.header).join('')
    );

    expect(result[1].slice(2)).to.be.equal(
      reverseEndianness(blockHeaders[6000 - 1].hash)
    );
  });

});