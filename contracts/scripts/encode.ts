import { ethers } from 'ethers';

function encodePacked(data: any[], types: string[]): string {
    const encoded = ethers.solidityPacked(types, data);
    return encoded;
}

console.log(
    encodePacked(
        [
            203610,
            "0xa12e1f2157c6f99469ccdb46ae68577273d3551f6a38d17ab304000000000000",
            "0xd09acdf9c9959a1754da9dae916e70bef9f131ad30ef8be2a503000000000000",
            "8825801199382903987726989797449454220615414953524072026210304"
        ],
        [
            'uint64',
            'bytes32',
            'bytes32',
            'uint256'
        ]
    )
)