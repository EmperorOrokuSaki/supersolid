use ic_exports::candid::CandidType;

#[derive(CandidType)]
pub enum RouterError {
    Unknown(String)
}