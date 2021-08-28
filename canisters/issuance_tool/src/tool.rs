use crate::management_canister::*;
use crate::types::*;
use candid::encode_args;
use ic_cdk::api::time;
use ic_cdk::export::candid::Principal;
use ic_cdk::{api, storage};

use candid::candid_method;
use ic_cdk_macros::*;
use std::string::String;
const CYCLES_PER_TOKEN: u64 = 2000_000_000_000;
static mut INITIALIZED: bool = false;
static mut OWNER: Principal = Principal::anonymous();
static mut STORAGE_CANISTER_ID: Principal = Principal::anonymous();
static mut FEE_TOKEN_ID: Principal = Principal::anonymous();
static mut FEE: u128 = 0u128;

#[update(name = "initialize")]
#[candid_method(update, rename = "initialize")]
fn initialize(storage_canister_id: Principal) -> bool {
    if storage_canister_id == Principal::anonymous() {
        ic_cdk::trap("invalid storage canister id");
    }
    unsafe {
        if INITIALIZED != false {
            ic_cdk::trap("initialized");
        }

        INITIALIZED = true;
        STORAGE_CANISTER_ID = storage_canister_id;
        OWNER = api::caller();

        api::print("initialized");
    }
    true
}

#[update(name = "setIssueFee")]
#[candid_method(update, rename = "setIssueFee")]
fn set_issue_fee(token_id: Principal, fee: u128) {
    _must_initialized();
    _only_owner();
    unsafe {
        FEE_TOKEN_ID = token_id;
        FEE = fee;
    };
}

#[update(name = "uploadTokenWasm")]
#[candid_method(update, rename = "uploadTokenWasm")]
fn upload_token_wasm(args: StoreWASMArgs) {
    _must_initialized();
    _only_owner();
    let token_bytes = storage::get_mut::<WASMBytes>();
    token_bytes.token_wasm = Some(args.wasm_module);
}

#[update(name = "uploadTokenStorageWasm")]
#[candid_method(update, rename = "uploadTokenStorageWasm")]
fn upload_token_storage_wasm(args: StoreWASMArgs) {
    _must_initialized();
    _only_owner();
    let token_bytes = storage::get_mut::<WASMBytes>();
    token_bytes.storage_wasm = Some(args.wasm_module);
}

#[update(name = "issueToken")]
#[candid_method(update, rename = "issueToken")]
async fn issue_token(args: IssueTokenArgs) -> Result<IssueResult, String> {
    _must_initialized();
    let caller = api::caller();

    if caller == Principal::anonymous() {
        api::trap("invalid issuer")
    }

    api::print(format!("issue token caller is {}", caller.to_text()));
    let wasm_bytes = storage::get::<WASMBytes>();
    let wasm_module = match &wasm_bytes.token_wasm {
        None => return Err("invalid token wasm module".to_string()), //std::include_bytes!("../wasm/dft_rs_opt.wasm").to_vec(),
        Some(o) => o.clone(),
    };
    unsafe {
        if FEE_TOKEN_ID != Principal::anonymous() && FEE > 0 {
            _charge_token_issue_fee(
                args.sub_account.clone(),
                caller.to_text(),
                api::id().to_text(),
                FEE,
            )
            .await;
        }
    }

    let create_args = CreateCanisterArgs {
        cycles: CYCLES_PER_TOKEN,
        settings: CanisterSettings {
            controllers: Some(vec![caller.clone(), api::id()]),
            compute_allocation: None,
            memory_allocation: None,
            freezing_threshold: None,
        },
    };

    api::print("start issue token...");
    let create_result = create_canister_call(create_args).await?;
    let install_args = encode_args((
        args.sub_account,
        args.logo,
        args.name.to_string(),
        args.symbol.to_string(),
        args.decimals.clone(),
        args.total_supply.clone(),
        args.fee.clone(),
        Some(caller.clone()),
    ))
    .expect("Failed to encode arguments.");

    let token_id = create_result.canister_id.clone();
    api::print(format!("new token id : {}", token_id.to_string()));

    match install_canister(&create_result.canister_id, wasm_module, install_args).await {
        Ok(_) => {
            let create_storage_res =
                create_token_storage_canister(caller.clone(), create_result.canister_id.clone())
                    .await;

            if let Ok(stroage_canister_id) = create_storage_res {
                api::print("set token tx storage...");
                let _set_storage_res: Result<(bool,), _> = api::call::call(
                    token_id.clone(),
                    "setStorageCanisterID",
                    (stroage_canister_id.clone(),),
                )
                .await;
                match _set_storage_res {
                    Ok(_) => api::print("set token storage succeed"),
                    Err(err) => api::print(format!("set token storage failed{}", err.1)),
                }
            }

            unsafe {
                api::print(format!("save token caller is {}", caller.to_text()));
                _save_tokeninfo(TokenInfo {
                    issuer: caller.clone(),
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

            api::print(format!("callback:{}", create_result.canister_id));
            Ok(create_result)
        }
        Err(e) => {
            api::print(format!("install token wasm failed. details:{}", e));
            Err(e)
        }
    }
}

async fn create_token_storage_canister(
    caller: Principal,
    token_id: Principal,
) -> Result<Principal, String> {
    let wasm_bytes = storage::get::<WASMBytes>();
    let wasm_module = match &wasm_bytes.storage_wasm {
        Some(v) => v.clone(),
        None => return Err("invalid token storage wasm module".to_string()), //std::include_bytes!("../wasm/graphql_opt.wasm").to_vec();
    };

    let create_args = CreateCanisterArgs {
        cycles: CYCLES_PER_TOKEN,
        settings: CanisterSettings {
            controllers: Some(vec![caller, api::id()]),
            compute_allocation: None,
            memory_allocation: None,
            freezing_threshold: None,
        },
    };

    api::print("creating token storage...");
    let create_result = create_canister_call(create_args).await?;
    let install_args = encode_args((token_id.clone(),)).expect("Failed to encode arguments.");

    api::print(format!(
        "token storage canister id : {}",
        create_result.canister_id.clone().to_string()
    ));
    match install_canister(&create_result.canister_id, wasm_module, install_args).await {
        Ok(_) => {
            // set storage
            let _set_storage_res: Result<(bool,), _> = api::call::call(
                token_id,
                "setStorageCanisterID",
                (create_result.canister_id,),
            )
            .await;
            Ok(create_result.canister_id.clone())
        }
        Err(e) => {
            api::print(format!(
                "install token storage canister failed. details:{}",
                e
            ));
            Err(e)
        }
    }
}

#[query(name = "toolGraph")]
async fn token_graphql() -> Principal {
    unsafe { STORAGE_CANISTER_ID }
}

#[update(name = "graphql_mutation")]
async fn graphql_mutation(mutation_string: String, variables_json_string: String) -> String {
    _only_owner();
    unsafe {
        //call storage canister
        let _storage_res: Result<(String,), _> = api::call::call(
            STORAGE_CANISTER_ID,
            "graphql_mutation_custom",
            (mutation_string, variables_json_string),
        )
        .await;

        match _storage_res {
            Ok((res,)) => res,
            Err((_, emsg)) => {
                api::print(format!("graphql_mutation failed:{}", emsg));
                emsg
            }
        }
    }
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
            Ok((tx_res,)) => {
                match tx_res {
                    TransferResult::Ok(_) => {
                        api::print(format!("_charge_token_issue_fee:{}", value));
                    }
                    TransferResult::Err(e) => {
                        api::trap(format!("charge issue fee failed,details:{}", e).as_str());
                    }
                };
            }
            Err(_) => api::trap("charge issue fee failed"),
        }
    }
}

async fn _save_tokeninfo(token_info: TokenInfo) {
    let vals = "{}".to_string();
    let muation = format!(
        r#"mutation {{ 
                        createTokenInfo(input: {{ 
                            token_id:"{0}",
                            issuer:"{1}",
                            name:"{2}",
                            symbol:"{3}",
                            decimals:{4},
                            total_supply:"{5}",
                            fee_lowest:"{6}",
                            fee_rate:"{7}",
                            timestamp:"{8}",
                            }}) 
                            {{ id }} 
                           }}"#,
        token_info.token_id,
        token_info.issuer.to_string(),
        token_info.name,
        token_info.symbol,
        token_info.decimals.to_string(),
        token_info.total_supply.to_string(),
        token_info.fee.lowest.to_string(),
        token_info.fee.rate.to_string(),
        token_info.timestamp.to_string(),
    );
    api::print("saving tokeninfo ...");
    unsafe {
        let _storage_res: Result<(String,), _> = api::call::call(
            STORAGE_CANISTER_ID,
            "graphql_mutation_custom",
            (muation, vals),
        )
        .await;
        match _storage_res {
            Ok(res) => api::print(format!("_save_tokeninfo result:{}", res.0)),
            Err((code, emsg)) => api::print(format!("_save_tokeninfo failed:{}", emsg)),
        }
    }
}

fn _only_owner() {
    unsafe {
        if OWNER != api::caller() {
            api::trap(
                format!(
                    "caller is not the owner,caller:{},owner:{}",
                    api::caller().to_text(),
                    OWNER.to_text()
                )
                .as_str(),
            );
        }
    }
}

fn _must_initialized() {
    unsafe {
        if INITIALIZED != true {
            api::trap("uninitialized");
        }
    }
}

#[pre_upgrade]
fn pre_upgrade() {
    unsafe {
        let wasm_bytes = storage::get::<WASMBytes>();
        let token_wasm = match &wasm_bytes.token_wasm {
            Some(t) => t.clone(),
            None => [].to_vec(),
        };
        let storage_wasm = match &wasm_bytes.storage_wasm {
            Some(t) => t.clone(),
            None => [].to_vec(),
        };
        let stable = StableStorage {
            initialized: INITIALIZED,
            owner: OWNER,
            storage_canister_id: STORAGE_CANISTER_ID,
            fee_token_id: FEE_TOKEN_ID,
            fee: FEE,
            token_wasm,
            storage_wasm,
        };
        match storage::stable_save((stable,)) {
            Ok(_) => (),
            Err(candid_err) => {
                ic_cdk::trap(&format!(
                    "An error occurred when saving to stable memory (pre_upgrade): {}",
                    candid_err
                ));
            }
        };
    }
}

#[post_upgrade]
fn post_upgrade() {
    if let Ok((storage,)) = storage::stable_restore::<(StableStorage,)>() {
        unsafe {
            INITIALIZED = storage.initialized;
            let wasm_bytes = storage::get_mut::<WASMBytes>();
            if storage.token_wasm.len() > 0 {
                wasm_bytes.token_wasm = Some(storage.token_wasm);
            }
            if storage.storage_wasm.len() > 0 {
                wasm_bytes.storage_wasm = Some(storage.storage_wasm);
            }
        }
    }
}
