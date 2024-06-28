use std::time::Duration;

use ic_exports::{ic_cdk_timers::set_timer, ic_kit::ic::spawn};
use serde_json::json;

use crate::{
    evm_rpc::{RpcApi, RpcService}, state::{CHAIN_RPCS, ROUTER_KEY, RPC_CANISTER}, types::EthCallResponse, utils::{decode_request, decode_response, handle_rpc_response}
};

pub async fn check_chains() {
    let rpcs: Vec<(u64, String)> =
        CHAIN_RPCS.with(|rpcs| rpcs.borrow().clone().into_iter().collect());
    for (chain_id, rpc) in rpcs {
        set_timer(Duration::from_secs(1), move || {
            spawn(async move {
                check_chain(chain_id, rpc).await;
            });
        });
    }
}

pub async fn check_chain(chain_id: u64, rpc: String) {
    // get current block number
    let current_block_number = eth_get_block_number(chain_id, rpc.clone()).await;
}

pub async fn eth_get_block_number(chain_id: u64, rpc: String) -> u64 {
    let router_key = ROUTER_KEY.with(|key| key.borrow().clone());
    let rpc_canister = RPC_CANISTER.with(|canister| canister.borrow().clone());
    let rpc_service = RpcService::Custom(RpcApi {
        url: rpc,
        headers: None,
    });
    let json_data = json!({
            "id": 1,
            "jsonrpc": "2.0",
            "params": [],
            "method": "eth_blockNumber"
    });
    
    match decode_request(
        rpc_canister
            .request(rpc_service, json_data.to_string(), 500000, 10000000)
            .await,
    ) {
        Ok(decoded_bytes) => {
            
        },
        Err()
    }
}
