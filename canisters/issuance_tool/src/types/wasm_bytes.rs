use candid::{CandidType, Deserialize};

#[derive(CandidType, Default, Clone, Deserialize)]
pub struct WASMBytes {
    pub token_wasm: Option<Vec<u8>>,
}
#[derive(CandidType, Deserialize)]
pub struct StoreWASMArgs {
    #[serde(with = "serde_bytes")]
    pub wasm_module: Vec<u8>,
}
