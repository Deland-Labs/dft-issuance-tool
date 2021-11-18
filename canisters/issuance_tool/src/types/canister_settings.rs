use candid::{CandidType, Deserialize, Nat, Principal};

#[derive(CandidType, Clone, Deserialize)]
pub struct CanisterSettings {
    // dfx versions <= 0.8.1 (or other wallet callers expecting version 0.1.0 of the wallet)
    // will set a controller (or not) in the the `controller` field:
    pub controller: Option<Principal>,

    // dfx versions >= 0.8.2 will set 0 or more controllers here:
    pub controllers: Option<Vec<Principal>>,

    pub compute_allocation: Option<Nat>,
    pub memory_allocation: Option<Nat>,
    pub freezing_threshold: Option<Nat>,
}

#[derive(CandidType, Deserialize, Debug, Eq, PartialEq)]
pub struct DefiniteCanisterSettingsArgs {
    pub controllers: Vec<Principal>,
    pub compute_allocation: candid::Nat,
    pub memory_allocation: candid::Nat,
    pub freezing_threshold: candid::Nat,
}

impl From<DefiniteCanisterSettingsArgs> for CanisterSettings {
    fn from(args: DefiniteCanisterSettingsArgs) -> Self {
        CanisterSettings {
            controllers: Some(args.controllers),
            compute_allocation: Some(args.compute_allocation),
            memory_allocation: Some(args.memory_allocation),
            freezing_threshold: Some(args.freezing_threshold),
            controller: None,
        }
    }
}
