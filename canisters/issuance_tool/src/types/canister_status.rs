use candid::{CandidType, Deserialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, CandidType)]
pub enum CanisterStatus {
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "stopping")]
    Stopping,
    #[serde(rename = "stopped")]
    Stopped,
}