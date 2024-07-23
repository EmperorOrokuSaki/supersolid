use crate::{
    evm_rpc::{MultiSendRawTransactionResult, RequestResult, RpcApi, RpcService, Service},
    signer::{get_canister_public_key, pubkey_bytes_to_address},
    state::*,
    timers::check_chains,
    types::{
        ChainState, LedgerKey, RouterError, RouterTxReceipt, ServiceRequest, UserBalances,
        ROOT_DERIVATION_PATH,
    },
    utils::send_raw_transaction,
};
use alloy_primitives::U256;
use candid::Nat;
use ic_canister::{generate_idl, query, update, Canister, Idl, PreUpdate};
use ic_exports::{
    candid::Principal,
    ic_cdk::{caller, print, spawn},
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
    pub fn start(&self, chains_tuple: Vec<(String, u64)>) {
        print("[INIT] Initializing the canister...");
        let mut chains: HashMap<u64, ChainState> = HashMap::new();

        for (rpc, chain_id) in chains_tuple {
            let (ledger, nonce) = match chain_id {
                8453 => {
                    // base
                    let mut ledger: HashMap<LedgerKey, UserBalances> = HashMap::new();
                    let mut swap_canister_ledger: UserBalances = HashMap::new();
                    swap_canister_ledger.insert(None, U256::from(25000000000000000 as u64));
                    ledger.insert(
                        LedgerKey::IcPrincipal(
                            Principal::from_str("zydig-qiaaa-aaaal-ajn6a-cai").unwrap(),
                        ),
                        swap_canister_ledger,
                    );
                    (ledger, 0)
                }
                42161 => {
                    // arbitrum
                    let mut ledger: HashMap<LedgerKey, UserBalances> = HashMap::new();
                    let mut swap_canister_ledger: UserBalances = HashMap::new();
                    swap_canister_ledger.insert(None, U256::from(25000000000000000 as u64));
                    ledger.insert(
                        LedgerKey::IcPrincipal(
                            Principal::from_str("zydig-qiaaa-aaaal-ajn6a-cai").unwrap(),
                        ),
                        swap_canister_ledger,
                    );
                    (ledger, 4)
                }
                _ => (HashMap::new(), 0),
            };
            let chain_state = ChainState {
                chain_id: chain_id,
                rpc: rpc,
                lock: false,
                last_checked_block: None,
                balance: U256::from(0),
                ledger,
                nonce,
            };
            chains.insert(chain_id, chain_state);
        }

        CHAINS.with(|rpcs| *rpcs.borrow_mut() = chains);
        RPC_CANISTER.with(|rpc_canister| {
            *rpc_canister.borrow_mut() =
                Service(Principal::from_str("7hfb6-caaaa-aaaar-qadga-cai").unwrap())
        });
        print("[INIT] Initialization is completed.");

        self.start_timers();
    }

    fn start_timers(&self) {
        print("[TIMER] Setting up timers...");
        set_timer(Duration::from_secs(1), || {
            spawn(async {
                print("[Timer] Setting public key...");
                let router_key = ROUTER_KEY.with(|key| key.borrow().clone());
                let pk: Vec<u8> =
                    get_canister_public_key(router_key, None, ROOT_DERIVATION_PATH).await;
                let public_key: String = pubkey_bytes_to_address(&pk);
                ROUTER_PUBLIC_KEY.with(|pk| *pk.borrow_mut() = public_key);
                print("[Timer] Public key is set.");

                print("[Timer] Initializing chain checks...");
                //check_chains().await;
                print("[Timer] First check is finished.");
            });
        });

        print("[Timer] Initializing chain checks in ten minute intervals...");
        // set_timer_interval(Duration::from_secs(10800), || {
        //     spawn(async {
        //         print("[Timer] Starting chain check...");
        //         check_chains().await;
        //         print("[Timer] Chain check cycle completed.");
        //     });
        // });
    }

    // #[update]
    // pub async fn poll(&self) {
    //     print("[Poll] polling...");
    //     check_chains().await;
    //     print("[Poll] polled.");
    // }

    #[update]
    pub async fn set_nonce(&mut self, nonce: u64, chain_id: u64) {
        CHAINS.with(|chains| {
            let mut binding = chains.borrow_mut();
            let chain = binding.get_mut(&chain_id).unwrap();
            chain.nonce = nonce;
        });
    }

    #[update]
    pub async fn withdraw_to(
        &mut self,
        destination_address: String,
        chain_id: u64,
        amount: u128,
        block_number: u64
    ) -> Result<(), RouterError> {
        let rpc_canister = RPC_CANISTER.with(|canister| canister.borrow().clone());

        let chain_state = CHAINS.with(|chains| {
            let mut binding = chains.borrow_mut();
            let chain = binding.get_mut(&chain_id).unwrap();

            chain.clone()
        });

        let block_number_nat = Nat::from(block_number);
        send_raw_transaction(
            chain_id,
            destination_address,
            vec![],
            amount,
            chain_state.nonce,
            ROOT_DERIVATION_PATH,
            &rpc_canister,
            block_number_nat,
            &chain_state.rpc,
            30_000_000_000,
        )
        .await?;

        CHAINS.with(|chains| {
            let mut binding = chains.borrow_mut();
            let chain = binding.get_mut(&chain_id).unwrap();
            chain.nonce += 1;
        });

        Ok(())
    }

    #[update]
    pub async fn send_request(
        &mut self,
        destination_chain_id: u64,
        destination_address: String,
        data: String,
        native_token_value: u128,
    ) -> Result<(), RouterError> {
        print("*/*/*/*/*/ Tx req received.");
        let rpc_canister = RPC_CANISTER.with(|canister| canister.borrow().clone());
        let chain_state = CHAINS.with(|chains| {
            let mut binding = chains.borrow_mut();
            let chain = binding.get_mut(&destination_chain_id).unwrap();

            let user_balance = chain
                .ledger
                .get_mut(&LedgerKey::IcPrincipal(caller()))
                .unwrap();
            let user_native_balance = user_balance.get_mut(&None).unwrap();
            if user_native_balance <= &mut U256::from(native_token_value) {
                return Err(RouterError::InsufficientFunds);
            } else {
                *user_native_balance -= U256::from(native_token_value);
                return Ok(chain.clone());
            }
        })?;

        print("*/*/*/*/*/ Starting to submit transaction.");

        // send_raw_transaction(
        //     destination_chain_id,
        //     destination_address,
        //     data.into_bytes(),
        //     native_token_value,
        //     chain_state.nonce,
        //     ROOT_DERIVATION_PATH,
        //     &rpc_canister,
        //     &chain_state.rpc,
        //     30_000_000_000,
        // )
        // .await?;

        CHAINS.with(|chains| {
            let mut binding = chains.borrow_mut();
            let chain = binding.get_mut(&destination_chain_id).unwrap();
            chain.nonce += 1;
        });

        Ok(())
    }

    #[update]
    pub fn add_request(
        &mut self,
        caller_identity: String,
        data: String,
        target_service: Principal,
    ) {
        SERVICE_REQUESTS.with(|services| {
            let mut map = services.borrow_mut();
            if let Some(vector) = map.get_mut(&target_service) {
                vector.push(ServiceRequest {
                    caller: caller_identity,
                    data,
                });
            } else {
                map.insert(
                    target_service,
                    vec![ServiceRequest {
                        caller: caller_identity,
                        data,
                    }],
                );
            }
        });
    }

    #[query]
    pub fn balance(&self, chain_index: u64) -> String {
        let balance_value: U256 =
            CHAINS.with(|chains| chains.borrow().get(&chain_index).unwrap().balance);
        balance_value.to_string()
    }

    #[query]
    pub fn get_chain_ledger(
        &self,
        chain_id: u64,
    ) -> Vec<(LedgerKey, Vec<(Option<String>, String)>)> {
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
        user: Option<LedgerKey>,
    ) -> u64 {
        CHAINS.with(|chains| {
            let binding = chains.borrow();
            let target = match user {
                None => LedgerKey::IcPrincipal(caller()),
                _ => user.unwrap(),
            };

            let chain_state: &ChainState = binding.get(&chain_id).unwrap(); // todo: we are assuming chain exists here, double check todo
            let user_ledger = chain_state.ledger.get(&target).unwrap(); // todo: we are assuming the user has an entry here, double check todo
            let user_balance = user_ledger.get(&token_address).unwrap(); //todo: we are assuming the user has token balance on this chain id, double check todo
            user_balance.to::<u64>()
        })
    }

    #[query]
    pub fn public_key(&self) -> String {
        ROUTER_PUBLIC_KEY.with(|pk| pk.borrow().clone())
    }

    #[query]
    pub fn poll_requests(&self, from: u64) -> Vec<ServiceRequest> {
        SERVICE_REQUESTS.with(|services| {
            let binding = services.borrow();
            // Safely get the vector of service requests
            if let Some(requests) = binding.get(&caller()) {
                // Ensure 'from' is within bounds
                if from as usize <= requests.len() {
                    // Return the requests starting from the 'from' index
                    requests[from as usize..].to_vec()
                } else {
                    // If 'from' is out of bounds, return an empty vector
                    Vec::new()
                }
            } else {
                // If no requests found for the caller, return an empty vector
                Vec::new()
            }
        })
    }

    #[query]
    pub fn poll_others_requests(&self, who: Principal) -> Vec<ServiceRequest> {
        SERVICE_REQUESTS.with(|services| {
            let binding = services.borrow();
            // Safely get the vector of service requests
            if let Some(requests) = binding.get(&who) {
                requests.clone()
            } else {
                // If no requests found for the caller, return an empty vector
                Vec::new()
            }
        })
    }

    pub fn idl() -> Idl {
        generate_idl!()
    }
}
