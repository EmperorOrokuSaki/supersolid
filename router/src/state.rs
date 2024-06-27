use std::{cell::RefCell, collections::HashMap};

use ic_exports::candid::Principal;

use crate::evm_rpc::Service;

thread_local! {
    pub static ETH_RPC_CANISTER: RefCell<Service> = RefCell::new(Service(Principal::anonymous()));
    pub static ETH_RPC_URL: RefCell<String> = RefCell::new("".to_string());
    pub static SOL_RPC_URL: RefCell<String> = RefCell::new("".to_string());
}
