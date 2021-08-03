#[warn(unused_must_use)]
use sudograph::graphql_database;
use sudograph::ic_cdk::export::Principal;
use sudograph::ic_cdk::storage::{stable_save,stable_restore}; 

graphql_database!("canisters/graphql/src/schema.graphql");

static mut OWNER:  Principal =     Principal::anonymous();
static mut TOOL_CANISTER_ID:  Principal =  Principal::anonymous();

#[derive(CandidType, Deserialize)]
struct Payload{
   owner:Principal,
   tool_canister_id:Principal
}

#[ init]
async fn init_custom() {
    unsafe {
        OWNER = sudograph::ic_cdk::caller();
        init().await
    }
}

#[sudograph::ic_cdk_macros::query(name = "graphql_query_custom")]
async fn graphql_query_custom(query: String, variables: String) -> String {
    unsafe {
        if  TOOL_CANISTER_ID != sudograph::ic_cdk::caller() { 
            sudograph::ic_cdk::trap("only allow owner");
         }
    }
    return graphql_query(query, variables).await;
}

#[sudograph::ic_cdk_macros::update(name = "graphql_mutation_custom")]
async fn graphql_mutation_custom(mutation_string: String, variables_json_string: String) -> String {
    unsafe {
        if  TOOL_CANISTER_ID != sudograph::ic_cdk::caller() { 
            sudograph::ic_cdk::trap("only allow owner");
         }
    }
    return graphql_mutation(mutation_string, variables_json_string).await;
}

#[sudograph::ic_cdk_macros::update]
async fn set_tool_canister_id(token: Principal) -> bool {
    unsafe {
        assert!(OWNER == sudograph::ic_cdk::caller());
        TOOL_CANISTER_ID = token;
    }
    true
}

#[sudograph::ic_cdk_macros::pre_upgrade]
async fn pre_upgrade_custom() {
    unsafe {
        let payload=Payload{
 owner:OWNER,
 tool_canister_id:TOOL_CANISTER_ID
        };
         stable_save((payload, )).unwrap();
    }
}

#[sudograph::ic_cdk_macros::post_upgrade]
async fn post_upgrade_costom() {
    let (payload, ): (
        Payload,
        
    ) = stable_restore().unwrap();
    unsafe {
        OWNER = payload.owner;
        TOOL_CANISTER_ID = payload.tool_canister_id;
    }
    post_upgrade().await
}
