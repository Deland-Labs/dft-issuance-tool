use candid::{CandidType, Deserialize};

use super::{CanisterStatus, DefiniteCanisterSettingsArgs};

#[derive(CandidType, Debug, Deserialize, Eq, PartialEq)]
pub struct CanisterStatusResultV2 {
    pub status: CanisterStatus,
    pub module_hash: Option<Vec<u8>>,
    pub controller: candid::Principal,
    pub settings: DefiniteCanisterSettingsArgs,
    pub memory_size: candid::Nat,
    pub cycles: candid::Nat,
    // this is for compat with Spec 0.12/0.13
    pub balance: Vec<(Vec<u8>, candid::Nat)>,
    pub freezing_threshold: candid::Nat,
}