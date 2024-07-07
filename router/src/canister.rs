use crate::{
    evm_rpc::{RequestResult, RpcApi, RpcService, Service},
    signer::{get_canister_public_key, pubkey_bytes_to_address},
    state::*,
    timers::check_chains,
    types::{ChainState, RouterError, RouterTxReceipt, UserBalances},
};
use alloy_primitives::U256;
use candid::Nat;
use ic_canister::{generate_idl, query, update, Canister, Idl, PreUpdate};
use ic_exports::{
    candid::Principal,
    ic_cdk::{print, spawn},
    ic_cdk_timers::{set_timer, set_timer_interval},
};
use serde_json::json;
use std::{collections::HashMap, str::FromStr, time::Duration};

#[derive(Canister)]
pub struct Supersolid {
    #[id]
    id: Principal,
}

impl PreUpdate for Supersolid {}

impl Supersolid {
    #[update]
    pub fn start(&self, rpc_principal: Principal, chains_tuple: Vec<(String, u64)>) {
        print("[INIT] Initializing the canister...");
        let mut chains: HashMap<u64, ChainState> = HashMap::new();

        for (rpc, chain_id) in chains_tuple {
            let chain_state = ChainState {
                chain_id: chain_id,
                rpc: rpc,
                lock: false,
                last_checked_block: None,
                balance: U256::from(0),
                ledger: HashMap::new(),
            };
            chains.insert(chain_id, chain_state);
        }

        CHAINS.with(|rpcs| *rpcs.borrow_mut() = chains);
        RPC_CANISTER.with(|rpc_canister| *rpc_canister.borrow_mut() = Service(rpc_principal));
        print("[INIT] Initialization is completed.");

        self.start_timers();
    }

    fn start_timers(&self) {
        print("[TIMER] Setting up timers...");
        set_timer(Duration::from_secs(1), || {
            spawn(async {
                print("[Timer] Setting public key...");
                let router_key = ROUTER_KEY.with(|key| key.borrow().clone());
                let pk: Vec<u8> = get_canister_public_key(router_key, None, None).await;
                let public_key: String = pubkey_bytes_to_address(&pk);
                ROUTER_PUBLIC_KEY.with(|pk| *pk.borrow_mut() = public_key);
                print("[Timer] Public key is set.");

                print("[Timer] Initializing chain checks...");
                check_chains().await;
                print("[Timer] First check is finished.");
            });
        });

        print("[Timer] Initializing chain checks in ten minute intervals...");
        set_timer_interval(Duration::from_secs(600), || {
            spawn(async {
                print("[Timer] Starting chain check...");
                check_chains().await;
                print("[Timer] Chain check cycle completed.");
            });
        });
    }

    #[update]
    pub async fn send_request(
        &mut self,
        destination_chain_id: u64,
        destination_address: String,
        data: String,
        native_token_value: Nat,
    ) -> Result<RouterTxReceipt, RouterError> {
        Err(RouterError::Unknown(String::from("Not implemented yet")))
    }

    #[query]
    pub fn balance(&self, chain_index: u64) -> String {
        let balance_value: U256 =
            CHAINS.with(|chains| chains.borrow().get(&chain_index).unwrap().balance);
        balance_value.to_string()
    }

    #[query]
    pub fn get_chain_ledger(&self, chain_id: u64) -> Vec<(String, Vec<(Option<String>, String)>)> {
        CHAINS.with(|chains| {
            let binding = chains.borrow();
            let chain_state: &ChainState = binding.get(&chain_id).unwrap(); // todo: we are assuming chain exists here, double check todo
            chain_state
                .ledger
                .iter()
                .map(|(user_addr, balances)| {
                    (
                        user_addr.clone(),
                        balances
                            .iter()
                            .map(|(token_addr, balance)| (token_addr.clone(), balance.to_string()))
                            .collect(),
                    )
                })
                .collect()
        })
    }

    #[query]
    pub fn get_user_balance(
        &self,
        chain_id: u64,
        token_address: Option<String>,
        user: String,
    ) -> String {
        CHAINS.with(|chains| {
            let binding = chains.borrow();
            let chain_state: &ChainState = binding.get(&chain_id).unwrap(); // todo: we are assuming chain exists here, double check todo
            let user_ledger = chain_state.ledger.get(&user).unwrap(); // todo: we are assuming the user has an entry here, double check todo
            let user_balance = user_ledger.get(&token_address).unwrap(); //todo: we are assuming the user has token balance on this chain id, double check todo
            user_balance.to_string()
        })
    }

    #[query]
    pub fn public_key(&self) -> String {
        ROUTER_PUBLIC_KEY.with(|pk| pk.borrow().clone())
    }

    pub fn idl() -> Idl {
        generate_idl!()
    }
}
