use crate::types::*;
use ic_cdk::api;
use ic_cdk::export::candid::{CandidType, Principal};
use serde::Deserialize;

// pub async fn create_canister_call(args: CreateCanisterArgs) -> Result<CreateResult, String> {
//     #[derive(CandidType)]
//     struct In {
//         settings: Option<CanisterSettings>,
//     }
//     let in_arg = In {
//         settings: Some(args.settings),
//     };

//     let (create_result,): (CreateResult,) = match api::call::call_with_payment(
//         Principal::management_canister(),
//         "create_canister",
//         (in_arg,),
//         args.cycles,
//     )
//     .await
//     {
//         Ok(x) => x,
//         Err((code, msg)) => {
//             return Err(format!(
//                 "An error happened during the call: {}: {}",
//                 code as u8, msg
//             ))
//         }
//     };

//     Ok(create_result)
// }

pub async fn get_canister_status(
    canister_id: &Principal,
) -> Result<CanisterStatusResultV2, String> {
    let (status, ): (CanisterStatusResultV2, ) = match api::call::call(
        Principal::management_canister(),
        "canister_status",
        (StatusRequest {
            canister_id: canister_id.clone(),
        }, ),
    )
        .await
    {
        Ok(x) => x,
        Err((code, msg)) => {
            return Err(format!(
                "An error happened during the call: {}: {}",
                code as u8, msg
            ));
        }
    };
    Ok(status)
}

pub async fn install_canister(
    canister_id: &Principal,
    wasm_module: Vec<u8>,
    args: Vec<u8>,
) -> Result<(), String> {
    // Install Wasm
    #[derive(CandidType, Deserialize)]
    enum InstallMode {
        #[serde(rename = "install")]
        Install,
        #[serde(rename = "reinstall")]
        Reinstall,
        #[serde(rename = "upgrade")]
        Upgrade,
    }

    #[derive(CandidType, Deserialize)]
    struct CanisterInstall {
        mode: InstallMode,
        canister_id: Principal,
        #[serde(with = "serde_bytes")]
        wasm_module: Vec<u8>,
        #[serde(with = "serde_bytes")]
        arg: Vec<u8>,
    }

    let install_config = CanisterInstall {
        mode: InstallMode::Install,
        canister_id: canister_id.clone(),
        wasm_module: wasm_module.clone(),
        arg: args,
    };

    match api::call::call(
        Principal::management_canister(),
        "install_code",
        (install_config, ),
    )
        .await
    {
        Ok(x) => x,
        Err((code, msg)) => {
            return Err(format!(
                "An error happened during the call: {}: {}",
                code as u8, msg
            ));
        }
    };
    Ok(())
}

pub async fn update_settings_call(
    args: UpdateSettingsArgs
) -> Result<(), String> {
    match api::call::call(Principal::management_canister(), "update_settings", (args, )).await {
        Ok(x) => x,
        Err((code, msg)) => {
            return Err(format!(
                "An error happened during the call: {}: {}",
                code as u8, msg
            ));
        }
    };
    Ok(())
}
