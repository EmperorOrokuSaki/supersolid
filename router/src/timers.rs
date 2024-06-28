use std::{ops::Deref, time::Duration};

use alloy_primitives::U256;
use ic_exports::{ic_cdk_timers::set_timer, ic_kit::ic::spawn};
use serde_json::json;

use crate::{
    evm_rpc::{RpcApi, RpcService},
    state::{CHAINS, ROUTER_KEY, ROUTER_PUBLIC_KEY, RPC_CANISTER},
    types::{ChainState, EthCallResponse},
    utils::{decode_request, decode_response},
};

pub async fn check_chains() {
    let chains: Vec<(u64, ChainState)> =
        CHAINS.with(|chains| chains.borrow().clone().into_iter().collect());
    for (index, chain) in chains {
        set_timer(Duration::from_secs(1), move || {
            spawn(async move {
                check_chain(index, chain).await;
            });
        });
    }
}

pub async fn check_chain(index: u64, state: ChainState) {
    // get current block number
    //let current_block_number = eth_get_block_number(&state.rpc).await;
}

pub async fn eth_get_balance(rpc: &str) -> U256 {
    let router_public_key = ROUTER_PUBLIC_KEY.with(|key| key.borrow().clone());
    let rpc_canister = RPC_CANISTER.with(|canister| canister.borrow().clone());
    let rpc_service = RpcService::Custom(RpcApi {
        url: rpc.to_string(),
        headers: None,
    });
    let json_data = json!({
            "id": 1,
            "jsonrpc": "2.0",
            "params": [
        router_public_key,
        "latest"
            ],
            "method": "eth_getBalance"
    });

    match decode_request(
        rpc_canister
            .request(rpc_service, json_data.to_string(), 500000, 10000000)
            .await,
    ) {
        Ok(decoded_bytes) => {
            U256::from_be_slice(&decoded_bytes)
        },
        Err(a) => unreachable!() // todo
    }
}

// pub async fn eth_get_block_number(rpc: &str) -> u64 {
//     let router_key = ROUTER_KEY.with(|key| key.borrow().clone());
//     let rpc_canister = RPC_CANISTER.with(|canister| canister.borrow().clone());
//     let rpc_service = RpcService::Custom(RpcApi {
//         url: rpc.to_string(),
//         headers: None,
//     });
//     let json_data = json!({
//             "id": 1,
//             "jsonrpc": "2.0",
//             "params": [],
//             "method": "eth_blockNumber"
//     });

//     match decode_request(
//         rpc_canister
//             .request(rpc_service, json_data.to_string(), 500000, 10000000)
//             .await,
//     ) {
//         Ok(decoded_bytes) => {

//         },
//         Err()
//     }
// }
