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
    
    let blockHashes = (await evmVerifier.verify(
      ethers.ZeroHash,
      '0xFFFF0000000000000000000000000000000000000000000000000000',
      '0x' + blockHeaders.slice(0, 2015).map((h:Header) => h.header).join('')
    ));

    for (let i = 0; i < blockHashes.length; i++) {
      expect(blockHashes[i].slice(2)).to.be.equal(
        reverseEndianness(blockHeaders[i].hash)
      );
    }
  });

  it('Should verify block header sequence with retargeting', async function () {
    const evmVerifier = await loadFixture(deployFixture);
    const blockHeaders:Array<Header> = await loadHeaders(201600, 211600);

    const result = await evmVerifier.verifyWithRetargeting(
      201600 + 2010 - 1,
      '0x' + reverseEndianness(blockHeaders[2010 - 1].hash), // parent block hash
      '0x' + reverseEndianness(blockHeaders[0].hash), // start period hash
      '0x057e080000000000000000000000000000000000000000000000',
      '0x' + blockHeaders[0].header + blockHeaders[2015].header + blockHeaders.slice(2010, 2010 + 2016).map((h:Header) => h.header).join('')
    );
    const blockHashes = result[0];
    const nextTarget = result[1];

    expect(ethers.toBeHex(nextTarget)).to.be.equal('0x0575ef0000000000000000000000000000000000000000000000');

    for (let i = 0; i < blockHashes.length; i++) {
      expect(blockHashes[i].slice(2)).to.be.equal(
        reverseEndianness(blockHeaders[2010 + i].hash)
      );
    }
  });

  it('Should verify block header sequence with retargeting (skip retarget)', async function () {
    const evmVerifier = await loadFixture(deployFixture);
    const blockHeaders:Array<Header> = await loadHeaders(201600, 211600);

    expect(evmVerifier.verifyWithRetargeting(
      201600,
      '0x' + reverseEndianness(blockHeaders[0].hash), // parent block hash
      '0x' + reverseEndianness(blockHeaders[0].hash), // start period hash
      '0x057e080000000000000000000000000000000000000000000000',
      '0x' + blockHeaders[0].header + blockHeaders[2015].header + blockHeaders.slice(1, 2000).map((h:Header) => h.header).join('')
    )).to.be.revertedWithCustomError(evmVerifier, 'InvalidHeadersInput()');
  });

});