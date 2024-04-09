import {
  JSONRPCClient,
  JSONRPCRequest,
  TypedJSONRPCClient
} from "json-rpc-2.0";
import dotenv from "dotenv"
import fs from "fs";

dotenv.config();

type BitcoinMethods = {
  getblockhash(params: [ number ]): string;
  getblockheader(params: [ string, boolean ]): string;
};

const arrayRange = (n: number, k: number) => Array.from({ length: (k - n) }, (_, i) => n + i);

async function promiseAllInBatches<T, I>(
  task: (item: I) => PromiseLike<T>, 
  items: I[], 
  batchSize: number = 10
): Promise<T[]> {
    let position = 0;
    let results: T[] = [];
    while (position < items.length) {
        const itemsForBatch = items.slice(position, position + batchSize);
        results = [...results, ...await Promise.all(itemsForBatch.map(item => task(item)))];
        position += batchSize;
    }
    return results;
}

const client: TypedJSONRPCClient<BitcoinMethods> = new JSONRPCClient((jsonRPCRequest: JSONRPCRequest) => {
  fetch(process.env.BITCOIN_RPC_URL || '', {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(jsonRPCRequest),
  }).then((response) => {
    if (response.status === 200) {
      return response
        .json()
        .then((jsonRPCResponse) => client.receive(jsonRPCResponse));
    } else if (jsonRPCRequest.id !== undefined) {
      return Promise.reject(new Error(response.statusText));
    }
  });
});

async function fetchBlocks(startHeight: number, endHeight: number, filePath: string): Promise<void> {
  const blockHeights = arrayRange(startHeight, endHeight);

  const blockHashes = await promiseAllInBatches<string, number>(
    (height) => client.request("getblockhash", [ height ]),
    blockHeights
  );

  const blockHeaders = await promiseAllInBatches<string, string>(
    (blockHash) => client.request("getblockheader", [ blockHash, false ]),
    blockHashes
  );

  const json = JSON.stringify(
    blockHeaders.map((header, i) => ({
      height: blockHeights[i], 
      hash: blockHashes[i],
      header
    })),
    null,
    2
  );
  fs.writeFileSync(filePath, json);

  console.log(`Fetched and saved ${blockHeaders.length} block headers to ${filePath}`);
}

const startHeight = 201600;
const endHeight = 201600 + 10000;
const filePath = `${__dirname}/fixtures/headers-${startHeight}-${endHeight}.json`;

fetchBlocks(startHeight, endHeight, filePath)
    .catch((error) => console.error(error));