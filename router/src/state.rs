use std::{cell::RefCell, collections::HashMap};

use ic_exports::{
    candid::Principal,
    ic_cdk::api::management_canister::ecdsa::{EcdsaCurve::Secp256k1, EcdsaKeyId},
};

use crate::evm_rpc::Service;

thread_local! {
    pub static RPC_CANISTER: RefCell<Service> = RefCell::new(Service(Principal::anonymous()));
    pub static CHAIN_RPCS: RefCell<HashMap<u64, String>> = RefCell::new(HashMap::new());
    pub static ETH_RPC_URL: RefCell<String> = RefCell::new("".to_string());
    pub static SOL_RPC_URL: RefCell<String> = RefCell::new("".to_string());
    pub static ROUTER_KEY: RefCell<EcdsaKeyId> = RefCell::new(EcdsaKeyId { curve: Secp256k1, name: "key_1".to_string() })
}
