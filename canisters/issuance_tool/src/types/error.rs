use candid::{CandidType, Deserialize};
use thiserror::Error;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, CandidType, Deserialize, Error)]
pub enum ToolError {
    #[error("Call it anonymous is not allow")]
    NotAllowAnonymous,
    #[error("Caller is not the owner")]
    OnlyOwnerAllowCallIt,
    #[error("Invalid token wasm module")]
    InvalidTokenWasmModule,
    #[error("Canister already installed")]
    CanisterAlreadyInstalled,
    #[error("Install token code failed, reason: {reason:?}")]
    InstallTokenCodeFailed { reason: String },
    #[error("Token not found")]
    TokenNotFound,
    #[error("Caller is not the controller of the token")]
    CallerIsNotControllerOfToken,
    #[error("error from remote, detail: {detail:?}")]
    Unknown { detail: String },
}

impl ToolError {
    pub(crate) fn code(&self) -> u32 {
        match self {
            ToolError::NotAllowAnonymous { .. } => 1,
            ToolError::OnlyOwnerAllowCallIt => 2,
            ToolError::InvalidTokenWasmModule => 3,
            ToolError::CanisterAlreadyInstalled => 4,
            ToolError::TokenNotFound { .. } => 5,
            ToolError::InstallTokenCodeFailed { .. } => 6,
            ToolError::CallerIsNotControllerOfToken => 7,
            ToolError::Unknown { .. } => 10000
        }
    }
}

impl From<ToolError> for ActorError {
    fn from(error: ToolError) -> Self {
        ActorError {
            code: error.code(),
            message: error.to_string(),
        }
    }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, CandidType, Deserialize)]
pub struct ActorError {
    code: u32,
    message: String,
}

pub type CommonResult<T> = anyhow::Result<T, ToolError>;
pub type ActorResult<T> = Result<T, ActorError>;

pub fn to_actor_result<T>(result: CommonResult<T>) -> ActorResult<T> {
    match result {
        Ok(value) => Ok(value),
        Err(error) => Err(error.into()),
    }
}