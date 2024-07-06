use std::{collections::HashMap, ops::Deref, str::FromStr, time::Duration};

use alloy_primitives::U256;
use ic_exports::{
    ic_cdk::{print, trap},
    ic_cdk_timers::set_timer,
    ic_kit::ic::spawn,
};
use serde_json::json;

use crate::{
    evm_rpc::{RpcApi, RpcService},
    state::{CHAINS, ROUTER_KEY, ROUTER_PUBLIC_KEY, RPC_CANISTER},
    types::{
        AlchemyGetAssetTransfersResponse, ChainState, EthCallResponse, Transfer, UserBalances,
    },
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

pub async fn check_chain(key: u64, mut state: ChainState) {
    if state.lock {
        return;
    }
    state.lock = true;

    CHAINS.with(|chains| {
        let mut binding = chains.borrow_mut();
        let mutable_state = binding.get_mut(&key).unwrap();
        mutable_state.lock = true;
    });

    // get current block number
    let current_block_number = eth_get_block_number(&state.rpc).await;
    let public_key = ROUTER_PUBLIC_KEY.with(|pk| pk.borrow().clone());

    let erc20_transfers = if let Some(old_block_number) = state.last_checked_block {
        // check each block between current_block_number and old_block_number
        eth_get_transfers(&state.rpc, U256::from(old_block_number), None, &public_key).await
    } else {
        // check from block zero
        eth_get_transfers(&state.rpc, U256::from(0), None, &public_key).await
    };

    for transfer in erc20_transfers {
        print(&format!("[TRANSFER] {:#?}", transfer));

        CHAINS.with(|chains| {
            let mut binding = chains.borrow_mut();
            let mutable_state = binding.get_mut(&key).unwrap();
            // get user ledger
            // if user ledger doesn't exist create it
            // get token value
            // if token doesn't exist create it
            // if token exists add to the balance
            match mutable_state.ledger.get_mut(&transfer.from) {
                Some(balances) => {
                    let token_balance = balances
                        .get(&transfer.raw_contract.address)
                        .unwrap_or(&U256::ZERO);
                    let mut bytes = [0_u8, 32];
                    let _ = hex::decode_to_slice(
                        &transfer.raw_contract.value.unwrap()[2..],
                        &mut bytes as &mut [u8],
                    );
                    let new_balance = token_balance + U256::from_be_bytes(bytes);
                    balances.insert(transfer.raw_contract.address, new_balance);
                }
                None => {
                    let mut balances = UserBalances::new();
                    let mut bytes = [0_u8, 32];
                    let _ = hex::decode_to_slice(
                        &transfer.raw_contract.value.unwrap()[2..],
                        &mut bytes as &mut [u8],
                    );
                    balances.insert(transfer.raw_contract.address, U256::from_be_bytes(bytes));
                    mutable_state.ledger.insert(transfer.from, balances);
                }
            };
        });
    }

    print(&format!(
        "[QUERY BEGIN] Chain id {} balance...",
        state.chain_id
    ));
    let balance = eth_get_balance(&state.rpc).await;
    state.balance = balance;
    print(&format!(
        "[QUERY END] Chain id {} balance => {}",
        state.chain_id, balance
    ));

    state.lock = false;
    CHAINS.with(|chains| chains.borrow_mut().insert(key, state));
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
            .request(rpc_service, json_data.to_string(), 500000, 20_000_000_000)
            .await,
    ) {
        Ok(decoded_bytes) => U256::from_be_slice(&decoded_bytes),
        Err(a) => trap(&format!("{:#?}", a)), // todo
    }
}

pub async fn eth_get_transfers(
    rpc: &str,
    from_block: U256,
    to_block: Option<U256>,
    to_address: &str,
) -> Vec<Transfer> {
    let rpc_canister = RPC_CANISTER.with(|canister| canister.borrow().clone());
    let rpc_service = RpcService::Custom(RpcApi {
        url: rpc.to_string(),
        headers: None,
    });

    let to_block = if to_block.is_some() {
        format!("0x{}", hex::encode(to_block.unwrap().to_string()))
    } else {
        "latest".to_string()
    };

    let json_data = json!({
      "id": 1,
      "jsonrpc": "2.0",
      "method": "alchemy_getAssetTransfers",
      "params": [
        {
          "fromBlock": format!("0x{}", hex::encode(from_block.to_string())),
          "toBlock": to_block,
          "toAddress": to_address,
          "withMetadata": false,
          "excludeZeroValue": true,
          "maxCount": "0x3e8", // todo bump from 1000
          "category": [
            "erc20"
          ],
          "order": "asc"
        }
      ]
    });
    let response = decode_response(
        rpc_canister
            .request(rpc_service, json_data.to_string(), 500000, 20_000_000_000)
            .await,
    );

    match response {
        Ok(a) => match a {
            crate::evm_rpc::RequestResult::Ok(data) => {
                let parsed_trasfers: AlchemyGetAssetTransfersResponse =
                    serde_json::from_str(&data).unwrap();
                parsed_trasfers.result.transfers
            }
            crate::evm_rpc::RequestResult::Err(err) => trap(&format!("[ERROR] {:#?}", err)),
        },
        Err(err) => trap(&format!("[ERROR] {:#?}", err)),
    }
}

pub async fn eth_get_block_number(rpc: &str) -> U256 {
    let rpc_canister = RPC_CANISTER.with(|canister| canister.borrow().clone());
    let rpc_service = RpcService::Custom(RpcApi {
        url: rpc.to_string(),
        headers: None,
    });

    let json_data = json!({
            "id": 1,
            "jsonrpc": "2.0",
            "method": "eth_blockNumber"
    });

    match decode_request(
        rpc_canister
            .request(rpc_service, json_data.to_string(), 500000, 20_000_000_000)
            .await,
    ) {
        Ok(decoded_bytes) => U256::from_be_slice(&decoded_bytes),
        Err(a) => trap(&format!("{:#?}", a)), // todo
    }
}
