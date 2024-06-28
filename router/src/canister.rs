use crate::{evm_rpc::Service, state::*, timers::check_chains};
use ic_canister::{generate_idl, init, Canister, Idl, PreUpdate};
use ic_exports::{candid::Principal, ic_cdk_timers::set_timer_interval, ic_kit::ic::spawn};
use std::{collections::HashMap, time::Duration};

#[derive(Canister)]
pub struct Supersolid {
    #[id]
    id: Principal,
}

impl PreUpdate for Supersolid {}

impl Supersolid {
    // INITIALIZATION
    #[init]
    pub fn init(
        &mut self,
        rpc_principal: Principal,
        chain_rpcs_tuple: Vec<(u64, String)>,
        sol_rpc_url: String,
    ) {
        let mut chain_rpcs: HashMap<u64, String> = HashMap::new();

        for (chain_id, rpc_endpoint) in chain_rpcs_tuple {
            chain_rpcs.insert(chain_id, rpc_endpoint);
        }

        CHAIN_RPCS.with(|rpcs| *rpcs.borrow_mut() = chain_rpcs);
        SOL_RPC_URL.with(|rpc| *rpc.borrow_mut() = sol_rpc_url);
        RPC_CANISTER.with(|rpc_canister| *rpc_canister.borrow_mut() = Service(rpc_principal));

        set_timer_interval(Duration::from_secs(60), || {
            spawn(async move {
                check_chains().await;
            });
        });
    }

    pub fn idl() -> Idl {
        generate_idl!()
    }
}
