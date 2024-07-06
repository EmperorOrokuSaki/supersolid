use std::collections::HashMap;

use alloy_primitives::U256;
use ic_exports::candid::CandidType;
use serde::Deserialize;
use serde_json::Value;

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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlchemyGetAssetTransfersResponse {
    pub id: u64,
    pub jsonrpc: String,
    pub result: Result,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Result {
    pub transfers: Vec<Transfer>,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transfer {
    pub block_num: String,
    pub unique_id: String,
    pub hash: String,
    pub from: String,
    pub to: String,
    pub value: Option<f64>,
    #[serde(rename = "erc721TokenId")]
    pub erc721token_id: Value,
    #[serde(rename = "erc1155Metadata")]
    pub erc1155metadata: Value,
    pub token_id: Value,
    pub asset: Option<String>,
    pub category: String,
    pub raw_contract: RawContract,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawContract {
    pub value: Option<String>,
    pub address: Option<String>,
    pub decimal: Option<String>,
}

pub type DerivationPath = Vec<Vec<u8>>;

#[derive(Clone)]
pub struct ChainState {
    pub rpc: String,
    pub chain_id: u64,
    pub lock: bool,
    pub last_checked_block: Option<u64>,
    /// native token balance of the chain
    pub balance: U256,
    /// Key: UserAddress, Value: Hashmap<TokenAddress, TokenValue>
    pub ledger: HashMap<String, UserBalances>,
}

/// Key: TokenContractAddress, Value: UserBalance
pub type UserBalances = HashMap<Option<String>, U256>;
