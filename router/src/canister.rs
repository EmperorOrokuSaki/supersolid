use crate::{
    evm_rpc::Service, sol_api::check_solana, state::*
};
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
    pub fn init(&mut self, eth_rpc_principal: Principal, eth_rpc_url: String, sol_rpc_url: String) {
        set_timer_interval(Duration::from_secs(60), || {
            spawn(async move {
                check_solana().await;
            });
        });

        ETH_RPC_CANISTER
            .with(|rpc_canister| *rpc_canister.borrow_mut() = Service(eth_rpc_principal));
        ETH_RPC_URL.with(|rpc| *rpc.borrow_mut() = eth_rpc_url);
        SOL_RPC_URL.with(|rpc| *rpc.borrow_mut() = sol_rpc_url);
    }

    pub fn idl() -> Idl {
        generate_idl!()
    }
}
