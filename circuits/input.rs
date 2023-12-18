use std::env;

use ethers::types::H256;

use bitcoincore_rpc::{Client, Auth, RpcApi};
use bitcoincore_rpc::bitcoin::consensus::serialize;
use bitcoincore_rpc::bitcoin::hex::DisplayHex;

use crate::consts::HEADER_BYTES_LENGTH;

pub struct InputDataFetcher {
    pub url: String,
    pub user: String,
    pub pass: String
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

    pub fn get_update_headers_inputs<const UPDATE_HEADERS_COUNT: usize>(
        &mut self,
        prev_block_number: u64,
        prev_header_hash: H256,
    ) -> Vec<[u8; HEADER_BYTES_LENGTH]> {
      let mut update_headers_bytes: Vec<[u8; HEADER_BYTES_LENGTH]> = Vec::new();

      let rpc = Client::new(&self.url,
                        Auth::UserPass(self.user.to_string(),
                                       self.pass.to_string())).unwrap();

      for i in 0..UPDATE_HEADERS_COUNT + 1 {
          let hash = rpc.get_block_hash(prev_block_number + i as u64).unwrap();

          if i == 0 {
              assert_eq!(
                  prev_header_hash,
                  H256::from_slice(serialize(&hash).as_slice())
              );
          } else {
              let header = rpc.get_block_header(&hash).unwrap();
              update_headers_bytes.push(
                  serialize(&header).try_into().unwrap()
              );

              println!("header {}: {}", i, update_headers_bytes[i - 1].as_hex());
          }
      }

      update_headers_bytes
    }
}