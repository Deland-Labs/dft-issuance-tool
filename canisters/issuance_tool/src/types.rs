use ic_cdk::export::candid::{CandidType, Nat, Principal};
use serde::{Deserialize, Serialize};

pub type TransactionId = u128;
#[derive(CandidType, Clone, Deserialize)]
pub struct Subaccount(pub [u8; 32]);

#[derive(CandidType, Default, Clone, Deserialize)]
pub struct WASMBytes {
    pub token_wasm: Option<Vec<u8>>,
    pub storage_wasm: Option<Vec<u8>>,
}

#[derive(CandidType, Deserialize)]
pub struct StoreWASMArgs {
    #[serde(with = "serde_bytes")]
    pub wasm_module: Vec<u8>,
}

#[derive(CandidType, Clone, Deserialize)]
pub struct CanisterSettings {
    pub controllers: Option<Vec<Principal>>,
    pub compute_allocation: Option<Nat>,
    pub memory_allocation: Option<Nat>,
    pub freezing_threshold: Option<Nat>,
}
#[derive(CandidType, Clone, Deserialize)]
pub struct CreateCanisterArgs {
    pub cycles: u64,
    pub settings: CanisterSettings,
}

#[derive(CandidType, Deserialize)]
pub struct UpdateSettingsArgs {
    pub canister_id: Principal,
    pub settings: CanisterSettings,
}

#[derive(CandidType, Deserialize)]
pub struct CreateResult {
    pub canister_id: Principal,
}

pub type IssueResult = CreateResult;

#[derive(CandidType, Deserialize)]
pub struct IssueTokenArgs {
    pub sub_account: Option<Subaccount>,
    pub logo: Option<Vec<u8>>,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: u128,
    pub fee: Fee,
}

// Rate decimals = 8
// transferFee = cmp::max(lowest,amount * rate / 10^8)
#[derive(CandidType, Debug, Clone, Deserialize)]
pub struct Fee {
    pub lowest: u128,
    pub rate: u128,
}

#[derive(CandidType, Deserialize)]
pub struct TokenInfo {
    pub issuer: Principal,
    pub token_id: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: u128,
    pub fee: Fee,
    pub timestamp: u64,
}

#[derive(CandidType, Debug, Clone, Deserialize, Serialize)]
pub struct TransferResponse {
    pub txid: TransactionId,
    pub error: Option<Vec<String>>,
}

#[derive(CandidType, Debug, Clone, Deserialize, Serialize)]
pub enum TransferResult {
    //transfer succeed, but call failed & notify failed
    Ok(TransferResponse),
    Err(String),
}

/// Until the stable storage works better in the ic-cdk, this does the job just fine.
#[derive(CandidType, Deserialize)]
pub struct StableStorage {
    pub initialized: bool,
    pub owner: Principal,
    pub storage_canister_id: Principal,
    pub fee_token_id: Principal,
    pub fee: u128,
    pub token_wasm: Vec<u8>,
    pub storage_wasm: Vec<u8>,
}
