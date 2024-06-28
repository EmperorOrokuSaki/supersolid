use crate::{
    evm_rpc::Service,
    signer::{get_canister_public_key, pubkey_bytes_to_address},
    state::*,
    timers::check_chains,
    types::ChainState,
};
use alloy::signers::utils::public_key_to_address;
use alloy_primitives::U256;
use candid::Nat;
use ic_canister::{generate_idl, init, query, update, Canister, Idl, PreUpdate};
use ic_exports::{
    candid::Principal,
    ic_cdk_timers::{set_timer, set_timer_interval},
    ic_kit::ic::spawn,
};
use std::{collections::HashMap, str::FromStr, time::Duration};

#[derive(Canister)]
pub struct Supersolid {
    #[id]
    id: Principal,
}

impl PreUpdate for Supersolid {}

impl Supersolid {
    // INITIALIZATION
    #[init]
    pub fn init(&mut self, rpc_principal: Principal, chains_tuple: Vec<(u64, (String, u64))>) {
        let mut chains: HashMap<u64, ChainState> = HashMap::new();

        for (index, (rpc, chain_id)) in chains_tuple {
            let chain_state = ChainState {
                chain_id: chain_id,
                rpc: rpc,
                lock: false,
                last_checked_block: None,
                balance: U256::from(0),
            };
            chains.insert(chain_id, chain_state);
        }

        CHAINS.with(|rpcs| *rpcs.borrow_mut() = chains);
        RPC_CANISTER.with(|rpc_canister| *rpc_canister.borrow_mut() = Service(rpc_principal));

        set_timer_interval(Duration::from_secs(60), || {
            spawn(async move {
                let router_key = ROUTER_KEY.with(|key| key.borrow().clone());
                let pk : Vec<u8> = get_canister_public_key(router_key, None, None).await;
                let public_key: String = pubkey_bytes_to_address(&pk);
                ROUTER_PUBLIC_KEY.with(|pk| *pk.borrow_mut() = public_key);
                check_chains().await;
            });
        });
    }

    #[query]
    pub fn balance(&self, chain_index: u64) -> Nat {
        let balance_value: U256 =
            CHAINS.with(|chains| chains.borrow().get(&chain_index).unwrap().balance);
        Nat::from_str(&balance_value.to_string()).unwrap()
    }

    #[query]
    pub fn public_key(&self) -> String {
        ROUTER_PUBLIC_KEY.with(|pk| pk.borrow().clone())
    }

    #[update]
    pub fn set_pk(&self) {
        ROUTER_PUBLIC_KEY.with(|pk| *pk.borrow_mut() = "HELLO".to_string());
    }

    pub fn idl() -> Idl {
        generate_idl!()
    }
}
