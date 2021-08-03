use crate::management_canister::*;
use crate::types::*;
use candid::{decode_args, encode_args};
use ic_cdk::api::time;
use ic_cdk::export::candid::{CandidType, Nat, Principal};
use ic_cdk::{api, caller, id, storage};

use candid::candid_method;
use ic_cdk_macros::*;
use std::string::String;
const CYCLES_PER_TOKEN: u64 = 2000000000000;
static mut INITIALIZED: bool = false;
static mut OWNER: Principal = Principal::anonymous();
static mut FEE_TOKEN_ID: Principal = Principal::anonymous();

#[ic_cdk_macros::import(canister = "graphql")]
struct GraphQLCanister;

fn _only_owner() {
    unsafe {
        assert!(OWNER == api::caller(), "caller is not the owner");
    }
}

fn _must_initialized() {
    unsafe {
        assert!(INITIALIZED == true, "uninitialized");
    }
}

#[update(name = "initialize")]
#[candid_method(update, rename = "initialize")]
fn initialize() -> bool {
    unsafe {
        if INITIALIZED != false {
            ic_cdk::trap("initialized");
        }

        INITIALIZED = true;
        OWNER = caller();
    }
    true
}

#[update(name = "setFeeTokenID")]
#[candid_method(update, rename = "setFeeTokenID")]
fn set_fee_token_id(token_id: Principal) {
    _must_initialized();
    _only_owner();
    unsafe { FEE_TOKEN_ID = token_id };
}

#[update(name = "uploadTokenWasm")]
#[candid_method(update, rename = "uploadTokenWasm")]
fn upload_token_wasm(args: TokenStoreWASMArgs) {
    _must_initialized();
    _only_owner();
    let token_bytes = storage::get_mut::<WalletWASMBytes>();
    token_bytes.0 = Some(serde_bytes::ByteBuf::from(args.wasm_module));
}

#[update(name = "issueToken")]
#[candid_method(update, rename = "issueToken")]
async fn issue_token(args: IssueTokenArgs) -> Result<IssueResult, String> {
    _must_initialized();
    let wallet_bytes = storage::get::<WalletWASMBytes>();
    let wasm_module = match &wallet_bytes.0 {
        None => {
            ic_cdk::trap("No wasm module stored.");
        }
        Some(o) => o,
    };

    //_charge_token_issue_fee(args.subaccount,args.)

    let create_args = CreateCanisterArgs {
        cycles: CYCLES_PER_TOKEN,
        settings: CanisterSettings {
            controllers: Some(vec![caller(), id()]),
            compute_allocation: None,
            memory_allocation: None,
            freezing_threshold: None,
        },
    };

    let create_result = create_canister_call(create_args).await?;
    let install_args = encode_args((
        args.subaccount,
        args.logo,
        args.name.to_string(),
        args.symbol.to_string(),
        args.decimals.clone(),
        args.total_supply.clone(),
        args.fee.clone(),
    ))
    .expect("Failed to encode arguments.");

    match install_canister(
        &create_result.canister_id,
        wasm_module.clone().into_vec(),
        install_args,
    )
    .await
    {
        Ok(_) => {
            unsafe {
                _save_tokeninfo(TokenInfo {
                    issuer: caller(),
                    token_id: create_result.canister_id.to_string(),
                    name: args.name.to_string(),
                    symbol: args.symbol.to_string(),
                    decimals: args.decimals,
                    total_supply: args.total_supply,
                    fee: args.fee.clone(),
                    timestamp: time(),
                })
                .await;
            }
            Ok(create_result)
        }
        Err(e) => Err(e),
    }
}

#[query(name = "graphql_query")]
async fn graphql_query(query: String, variables: String) -> String {
    let result = GraphQLCanister::graphql_query_custom(query, variables).await;
    return result.0;
}

async fn _charge_token_issue_fee(
    spender_sub_account: Option<Subaccount>,
    from: String,
    to: String,
    value: u128,
) {
    unsafe {
        let result: Result<(TransferResult,), _> = api::call::call(
            FEE_TOKEN_ID,
            "transferFrom",
            (spender_sub_account, from, to, value),
        )
        .await;
        match result {
            Ok((tx_res, )) => {
                match tx_res {
                    TransferResult::Ok(txid, _) => {}
                    TransferResult::Err(e) => {
                        api::trap("charge issue fee failed");
                    }
                };
            }
            Err(_) => todo!(),
        }
    }
}

async fn _save_tokeninfo(token_info: TokenInfo) {
    let vals = "{}".to_string();
    let muation = format!(
        r#"mutation {{ 
                        createTokenInfo(input: {{ 
                            token_id:  "{0}",
                            issuer:"{1}",
                            name:"{2}",
                            symbol:"{3}",
                            decimals:"{4}",
                            total_supply:"{5}",
                            fee:"{6}",
                            timestamp:"{7}",
                            }}) 
                            {{ id }} 
                           }}"#,
        token_info.token_id,
        token_info.issuer.to_string(),
        token_info.name,
        token_info.symbol,
        token_info.decimals.to_string(),
        token_info.total_supply.to_string(),
        token_info.fee.to_string(),
        token_info.timestamp.to_string(),
    );
    let result = GraphQLCanister::graphql_mutation_custom(muation, vals).await;

    api::print(format!("_save_tokeninfo result:{}", result.0));
}
