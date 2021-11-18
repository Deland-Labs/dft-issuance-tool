use std::{cell::RefCell};
use crate::management_canister::*;
use crate::types::*;
use candid::encode_args;
use ic_cdk::api::time;
use ic_cdk::export::candid::Principal;
use ic_cdk::{api, storage};

use candid::candid_method;
use ic_cdk_macros::*;
use std::string::String;
use crate::tool::*;

thread_local! {
    static ISSUANCE_TOOL: RefCell<IssuanceTool> = RefCell::new(IssuanceTool::new());
}
#[query(name = "owner")]
#[candid_method(query, rename = "owner")]
fn owner() -> Principal {
    ISSUANCE_TOOL.with(|tool| {
        let tool = tool.borrow();
        tool.owner()
    })
}

#[update(name = "setOwner")]
#[candid_method(update, rename = "setOwner")]
fn set_owner(owner: Principal) -> ActorResult<bool> {
    ISSUANCE_TOOL.with(|tool| {
        let mut tool = tool.borrow_mut();
        tool.set_owner(&api::caller(), owner)?;
        Ok(true)
    })
}

#[update(name = "setCyclesPerToken")]
#[candid_method(update, rename = "setCyclesPerToken")]
fn set_cycles_per_token(cycles: u64) -> ActorResult<bool> {
    ISSUANCE_TOOL.with(|tool| {
        let mut tool = tool.borrow_mut();
        let caller = api::caller();
        tool.set_cycles_per_token(&caller, cycles)?;
        Ok(true)
    })
}

#[update(name = "uploadTokenWasm")]
#[candid_method(update, rename = "uploadTokenWasm")]
fn upload_token_wasm(args: StoreWASMArgs) -> ActorResult<bool> {
    ISSUANCE_TOOL.with(|tool| {
        let mut tool = tool.borrow_mut();
        let wasm_bytes = args.wasm_module;
        let caller = api::caller();
        tool.set_token_wasm(&caller, wasm_bytes)?;
        Ok(true)
    })
}

#[query(name = "tokenOf")]
#[candid_method(query, rename = "tokenOf")]
fn token_of(token_id: Principal) -> ActorResult<TokenInfo> {
    ISSUANCE_TOOL.with(|tool| {
        let tool = tool.borrow();
        to_actor_result(tool.get_token_by_id(&token_id))
    })
}

#[query(name = "tokens")]
#[candid_method(query, rename = "tokens")]
fn tokens(start: usize, size: usize) -> ActorResult<Vec<TokenInfo>> {
    ISSUANCE_TOOL.with(|tool| {
        let tool = tool.borrow();
        to_actor_result(tool.get_tokens(start, size))
    })
}

#[update(name = "issueToken")]
#[candid_method(update, rename = "issueToken")]
async fn issue_token(args: IssueTokenArgs) -> ActorResult<IssueResult> {
    let caller = api::caller();
    let tool_id = api::id();
    ISSUANCE_TOOL.with(|tool| {
        let tool = tool.borrow();
        tool.only_owner(&caller)
    })?;

    api::print(format!("issue token caller is {}", caller.to_text()));

    // get token wasm
    let token_wasm = ISSUANCE_TOOL.with(|tool| {
        let tool = tool.borrow();
        tool.get_token_wasm()
    })?;
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

    match get_canister_status(&args.canister_id).await {
        Ok(status) => {
            // check if the caller is the controller of the token
            if !status.settings.controllers.contains(&caller) {
                return Err(ToolError::CallerIsNotControllerOfToken.into());
            }
            match status.module_hash {
                Some(_) => {
                    return Err(ToolError::CanisterAlreadyInstalled.into());
                }
                None => match install_canister(&args.canister_id, token_wasm, install_args).await {
                    Ok(_) => {
                        let token_info = TokenInfo {
                            issuer: caller.clone(),
                            token_id: args.canister_id.clone(),
                            name: args.name.to_string(),
                            symbol: args.symbol.to_string(),
                            decimals: args.decimals,
                            total_supply: args.total_supply,
                            fee: args.fee.clone(),
                            timestamp: time(),
                        };

                        // add token info to IssuanceTool
                        ISSUANCE_TOOL.with(|tool| {
                            let mut tool = tool.borrow_mut();
                            tool.add_token(&caller, token_info)
                        })?;

                        // remove issuance tool id from token's controllers
                        let mut settings: CanisterSettings = status.settings.into();
                        let mut current_controllers = settings.controllers.unwrap().clone();
                        current_controllers.retain(|c| c != &tool_id);
                        settings.controllers = Some(current_controllers);

                        let update_settings_args = UpdateSettingsArgs {
                            canister_id: args.canister_id.clone(),
                            settings,
                        };

                        match update_settings_call(update_settings_args).await {
                            Ok(_) => {}
                            Err(e) =>
                                return Err(ToolError::Unknown { detail: e }.into())
                        };

                        Ok(IssueResult {
                            canister_id: args.canister_id.clone(),
                        })
                    }
                    Err(e) => {
                        Err(ToolError::InstallTokenCodeFailed { reason: e.to_string() }.into())
                    }
                },
            }
        }
        Err(e) => {
            return Err(ToolError::Unknown { detail: e }.into());
        }
    }
}

// fn get tool status
#[query(name = "getStatus")]
#[candid_method(query, rename = "getStatus")]
fn get_status() -> ActorResult<ToolStatus> {
    ISSUANCE_TOOL.with(|tool| {
        let tool = tool.borrow();
        let mut status = tool.get_status();
        // get available cycles
        status.cycles = api::canister_balance();
        Ok(status)
    })
}

#[pre_upgrade]
fn pre_upgrade() {
    // to payload
    let payload = ISSUANCE_TOOL.with(|tool| {
        let tool = tool.borrow();
        tool.to_payload()
    });

    match storage::stable_save((payload, )) {
        Ok(_) => (),
        Err(candid_err) => {
            ic_cdk::trap(&format!(
                "An error occurred when saving to stable memory (pre_upgrade): {:?}",
                candid_err
            ));
        }
    };
}

#[post_upgrade]
fn post_upgrade() {
    match storage::stable_restore::<(ToolPayload, )>() {
        Ok((payload, )) => {
            ISSUANCE_TOOL.with(|tool| {
                let mut tool = tool.borrow_mut();
                tool.load_from_payload(payload)
            });
        }
        Err(candid_err) => {
            ic_cdk::trap(&format!(
                "An error occurred when saving to stable memory (post_upgrade): {:?}",
                candid_err
            ));
        }
    }
}
candid::export_service!();

#[query(name = "__get_candid_interface_tmp_hack")]
#[candid_method(query, rename = "__get_candid_interface_tmp_hack")]
fn __export_did_tmp_() -> String {
    __export_service()
}
