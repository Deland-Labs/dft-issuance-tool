use super::{CanisterSettings, Fee, Subaccount};
use candid::{CandidType, Deserialize, Principal};

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
pub type StatusRequest = CreateResult;

#[derive(CandidType, Deserialize)]
pub struct IssueTokenArgs {
    pub canister_id: Principal,
    pub sub_account: Option<Subaccount>,
    pub logo: Option<Vec<u8>>,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: u128,
    pub fee: Fee,
}
