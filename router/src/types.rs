use ic_exports::candid::CandidType;
use serde::Deserialize;

#[derive(CandidType)]
pub enum RouterError {
    Unknown(String)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EthCallResponse {
    pub id: u64,
    pub jsonrpc: String,
    pub result: String,
}