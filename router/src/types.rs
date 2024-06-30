use alloy_primitives::U256;
use ic_exports::candid::CandidType;
use serde::Deserialize;

use crate::evm_rpc::RpcError;

#[derive(CandidType, Debug)]
pub enum RouterError {
    Unknown(String),
    Rpc(RpcError),
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EthCallResponse {
    pub id: u64,
    pub jsonrpc: String,
    pub result: String,
}

pub type DerivationPath = Vec<Vec<u8>>;

#[derive(Clone)]
pub struct ChainState {
    pub rpc: String,
    pub chain_id: u64,
    pub lock: bool,
    pub last_checked_block: Option<u64>,
    pub balance: U256,
}
