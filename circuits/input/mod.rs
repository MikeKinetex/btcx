use std::env;

use bitcoincore_rpc::bitcoin::block::Header;
use bitcoincore_rpc::bitcoin::hashes::Hash;
use bitcoincore_rpc::bitcoin::BlockHash;
use ethers::types::H256;

use bitcoincore_rpc::bitcoin::consensus::serialize;
use bitcoincore_rpc::bitcoin::hex::DisplayHex;
use bitcoincore_rpc::{Auth, Client, RpcApi};

use crate::consts::HEADER_BYTES_LENGTH;

pub struct InputDataFetcher {
    pub url: String,
    pub user: String,
    pub pass: String,
}

impl Default for InputDataFetcher {
    fn default() -> Self {
        dotenv::dotenv().ok();

        let rpc_url = env::var("BITCOIN_RPC_URL").expect("BITCOIN_RPC_URL is not set in .env");
        let rpc_user = env::var("BITCOIN_RPC_USER").expect("BITCOIN_RPC_USER is not set in .env");
        let rpc_pass = env::var("BITCOIN_RPC_PASS").expect("BITCOIN_RPC_PASS is not set in .env");

        Self::new(&rpc_url, &rpc_user, &rpc_pass)
    }
}

impl InputDataFetcher {
    pub fn new(url: &str, user: &str, pass: &str) -> Self {
        Self {
            url: url.to_string(),
            user: user.to_string(),
            pass: pass.to_string(),
        }
    }

    fn get_client(&mut self) -> Client {
        Client::new(
            &self.url,
            Auth::UserPass(self.user.to_string(), self.pass.to_string()),
        )
        .unwrap()
    }

    pub fn get_header_by_height(&mut self, block_number: u64) -> Header {
        let rpc = self.get_client();
        let hash = rpc.get_block_hash(block_number as u64).unwrap();
        rpc.get_block_header(&hash).unwrap()
    }

    pub fn get_header_by_hash(&mut self, block_hash: H256) -> Header {
        let rpc = self.get_client();
        let hash = BlockHash::from_slice(block_hash.as_bytes()).unwrap();
        rpc.get_block_header(&hash).unwrap()
    }

    pub fn to_bytes(&mut self, header: &Header) -> [u8; HEADER_BYTES_LENGTH] {
        serialize(header).try_into().unwrap()
    }

    pub fn get_update_headers_inputs<const UPDATE_HEADERS_COUNT: usize>(
        &mut self,
        prev_block_number: u64,
        prev_header_hash: H256,
    ) -> Vec<[u8; HEADER_BYTES_LENGTH]> {
        let rpc = self.get_client();

        let mut update_headers_bytes: Vec<[u8; HEADER_BYTES_LENGTH]> = Vec::new();

        for i in 0..UPDATE_HEADERS_COUNT + 1 {
            let hash = rpc.get_block_hash(prev_block_number + i as u64).unwrap();

            if i == 0 {
                assert_eq!(
                    prev_header_hash,
                    H256::from_slice(serialize(&hash).as_slice())
                );
            } else {
                let header = rpc.get_block_header(&hash).unwrap();
                update_headers_bytes.push(serialize(&header).try_into().unwrap());

                println!("header {}: {}", i, update_headers_bytes[i - 1].as_hex());
            }
        }

        update_headers_bytes
    }
}
