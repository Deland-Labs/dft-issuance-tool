use crate::types::*;
use candid::{CandidType, Deserialize, Principal};
use std::collections::HashMap;

pub type TokenInfoMap = HashMap<Principal, TokenInfo>;

#[derive(CandidType, PartialOrd, Eq, PartialEq, Clone, Deserialize, Debug)]
pub struct TokenInfo {
    pub issuer: Principal,
    #[serde(rename = "tokenId")]
    pub token_id: Principal,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    #[serde(rename = "totalSupply")]
    pub total_supply: u128,
    pub fee: Fee,
    pub timestamp: u64,
}

#[derive(CandidType, Deserialize, PartialOrd, Eq, PartialEq, Clone, Debug)]
pub struct ToolPayload {
    pub owner: Principal,
    pub cycles_per_token: u64,
    pub token_wasm: Vec<u8>,
    pub tokens: Vec<(Principal, TokenInfo)>,
}

#[derive(CandidType, Deserialize)]
pub struct ToolStatus {
    pub owner: Principal,
    pub cycles_per_token: u64,
    pub cycles: u64,
    pub issued_token_count: u128,
}

pub struct IssuanceTool {
    pub owner: Principal,
    pub cycles_per_token: u64,
    pub token_wasm: Vec<u8>,
    pub tokens: TokenInfoMap,
}

impl IssuanceTool {
    pub fn new() -> Self {
        IssuanceTool {
            owner: Principal::anonymous(),
            cycles_per_token: 3000_000_000_000, // 3 T Cycles
            token_wasm: Vec::new(),
            tokens: TokenInfoMap::new(),
        }
    }

    // check if the caller is anonymous
    pub fn not_allow_anonymous(&self, caller: &Principal) -> CommonResult<()> {
        if caller == &Principal::anonymous() {
            return Err(ToolError::NotAllowAnonymous);
        }
        Ok(())
    }

    // check if the caller is the owner
    pub fn only_owner(&self, caller: &Principal) -> CommonResult<()> {
        self.not_allow_anonymous(caller)?;
        if &self.owner != caller {
            return Err(ToolError::OnlyOwnerAllowCallIt);
        }
        Ok(())
    }

    // get cycles per token
    pub fn cycles_per_token(&self) -> u64 {
        self.cycles_per_token
    }
    // get owner
    pub fn owner(&self) -> Principal {
        self.owner.clone()
    }
    // set owner
    pub fn set_owner(&mut self, caller: &Principal, owner: Principal) -> CommonResult<()> {
        self.not_allow_anonymous(caller)?;
        if self.owner != Principal::anonymous() {
            self.only_owner(caller)?;
        }
        self.owner = owner;
        Ok(())
    }

    // set cycles per token
    pub fn set_cycles_per_token(
        &mut self,
        caller: &Principal,
        cycles_per_token: u64,
    ) -> CommonResult<bool> {
        self.only_owner(caller)?;
        self.cycles_per_token = cycles_per_token;
        Ok(true)
    }

    // get tokens count
    pub fn get_token_count(&self) -> CommonResult<u128> {
        Ok(self.tokens.len() as u128)
    }

    // get token by id
    pub fn get_token_by_id(&self, token_id: &Principal) -> CommonResult<TokenInfo> {
        match self.tokens.get(token_id) {
            Some(token) => Ok(token.clone()),
            None => Err(ToolError::TokenNotFound),
        }
    }

    // get token with page parameters
    //  start_index: start index of the token list
    //  page_size: page size of the token list
    pub fn get_tokens(
        &self,
        start_index: usize,
        page_size: usize,
    ) -> CommonResult<Vec<TokenInfo>> {
        // max page size is 200
        let page_size = if page_size > 200 { 200 } else { page_size };
        let mut token_list = Vec::new();
        let mut index = 0;
        for token_info in self.tokens.values() {
            if index >= start_index as usize {
                token_list.push(token_info.clone());
            }
            index += 1;
            if index >= (start_index + page_size) as usize {
                break;
            }
        }
        Ok(token_list)
    }

    // add token
    pub fn add_token(&mut self, caller: &Principal, token_info: TokenInfo) -> CommonResult<()> {
        self.only_owner(caller)?;
        self.tokens.insert(token_info.token_id, token_info);
        Ok(())
    }

    // get token wasm
    pub fn get_token_wasm(&self) -> CommonResult<Vec<u8>> {
        // check wasm length
        if self.token_wasm.len() == 0 {
            return Err(ToolError::InvalidTokenWasmModule);
        } else {
            Ok(self.token_wasm.clone())
        }
    }

    // set token wasm
    pub fn set_token_wasm(&mut self, caller: &Principal, token_wasm: Vec<u8>) -> CommonResult<()> {
        self.only_owner(caller)?;
        self.token_wasm = token_wasm;
        Ok(())
    }

    // convert to ToolPayload
    pub fn to_payload(&self) -> ToolPayload {
        ToolPayload {
            owner: self.owner.clone(),
            cycles_per_token: self.cycles_per_token,
            token_wasm: self.token_wasm.clone(),
            tokens: self
                .tokens
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        }
    }

    // get ToolStatus
    pub fn get_status(&self) -> ToolStatus {
        ToolStatus {
            owner: self.owner.clone(),
            cycles_per_token: self.cycles_per_token,
            cycles: 0,
            issued_token_count: self.get_token_count().unwrap(),
        }
    }

    // load from ToolPayload
    pub fn load_from_payload(&mut self, payload: ToolPayload) {
        self.owner = payload.owner;
        self.cycles_per_token = payload.cycles_per_token;
        self.token_wasm = payload.token_wasm;
        self.tokens = payload.tokens.into_iter().map(|(k, v)| (k, v)).collect();
    }
}

//  IssuanceTool tests
#[cfg(test)]
mod tests {
    use super::*;
    use candid::Nat;
    use ic_types::Principal;

    // test get/set owner
    #[test]
    fn test_owner() {
        let mut tool = IssuanceTool::new();
        let owner =
            Principal::from_text("qupnt-ohzy3-npshw-oba2m-sttkq-tyawc-vufye-u5fbz-zb6yu-conr3-tqe")
                .unwrap();

        // set owner by call anonymous
        let result = tool.set_owner(&Principal::anonymous(), owner.clone());
        // check result is err
        assert!(result.is_err());
        // check error code is NotAllowAnonymous
        assert_eq!(result.unwrap_err(), ToolError::NotAllowAnonymous);
        // check owner is anonymous
        assert_eq!(tool.owner(), Principal::anonymous());

        // set owner by owner
        let result = tool.set_owner(&owner, owner.clone());
        // check result is ok
        assert!(result.is_ok());
        // check owner is owner
        assert_eq!(tool.owner(), owner);
        let new_owner =
            Principal::from_text("czjfo-ddpvm-6sibl-6zbox-ee5zq-bx3hc-e336t-s6pka-dupmy-wcxqi-fae")
                .unwrap();
        // set owner to new_owner
        let result = tool.set_owner(&new_owner, new_owner.clone());
        // check result is error
        assert!(result.is_err());
        // the error code is OnlyOwnerAllowCallIt
        assert_eq!(result.unwrap_err(), ToolError::OnlyOwnerAllowCallIt);

        // set owner to new_owner
        let result = tool.set_owner(&owner, new_owner.clone());
        // check result is ok
        assert!(result.is_ok());
        // check owner is new_owner
        assert_eq!(tool.owner(), new_owner);

        // check default value of cycles_per_token
        assert_eq!(tool.cycles_per_token(), 3000_000_000_000);
    }

    // test set cycles per token
    #[test]
    fn test_cycles_per_token() {
        let mut tool = IssuanceTool::new();
        let owner =
            Principal::from_text("qupnt-ohzy3-npshw-oba2m-sttkq-tyawc-vufye-u5fbz-zb6yu-conr3-tqe")
                .unwrap();
        // set cycles by anonymous will fail
        let result = tool.set_cycles_per_token(&Principal::anonymous(), 1);
        // check result is err
        assert!(result.is_err());
        // check error code is NotAllowAnonymous
        assert_eq!(result.unwrap_err(), ToolError::NotAllowAnonymous);

        // set cycles by not owner will fail
        let new_owner =
            Principal::from_text("czjfo-ddpvm-6sibl-6zbox-ee5zq-bx3hc-e336t-s6pka-dupmy-wcxqi-fae")
                .unwrap();
        let result = tool.set_cycles_per_token(&new_owner, 1);
        // check result is err
        assert!(result.is_err());
        // check error code is OnlyOwnerAllowCallIt
        assert_eq!(result.unwrap_err(), ToolError::OnlyOwnerAllowCallIt);
        let result = tool.set_owner(&owner, owner.clone());
        assert!(result.is_ok());
        let result = tool.set_cycles_per_token(&owner, 100);
        assert!(result.is_ok());
        assert_eq!(tool.cycles_per_token, 100);
    }

    // test add token
    #[test]
    fn test_add_token() {
        let mut tool = IssuanceTool::new();
        let owner =
            Principal::from_text("qupnt-ohzy3-npshw-oba2m-sttkq-tyawc-vufye-u5fbz-zb6yu-conr3-tqe")
                .unwrap();
        let new_owner =
            Principal::from_text("czjfo-ddpvm-6sibl-6zbox-ee5zq-bx3hc-e336t-s6pka-dupmy-wcxqi-fae")
                .unwrap();
        let result = tool.set_owner(&owner, owner.clone());
        assert!(result.is_ok());
        let token_id = Principal::from_text("g7cye-cyaaa-aaaak-aaa5a-cai").unwrap();

        // build a instance of TokenInfo

        let token_info = TokenInfo {
            issuer: owner.clone(),
            token_id: token_id.clone(),
            name: "test".to_string(),
            symbol: "TST".to_string(),
            decimals: 18,
            total_supply: 100,
            fee: Fee {
                minimum: Nat::from(1),
                rate: Nat::from(10000),
            },
            timestamp: 0,
        };
        let result = tool.add_token(&owner, token_info.clone());
        assert!(result.is_ok());
        assert_eq!(tool.tokens.len(), 1);
        // get token by id, then check token info
        let token = tool.get_token_by_id(&token_id).unwrap();
        assert_eq!(token.token_id, token_id);
        assert_eq!(token.name, "test".to_string());
        assert_eq!(token.symbol, "TST".to_string());
        assert_eq!(token.decimals, 18);
        assert_eq!(token.total_supply, 100);
        assert_eq!(token.fee.minimum, Nat::from(1));
        assert_eq!(token.fee.rate, Nat::from(10000));
        assert_eq!(token.timestamp, 0);
        assert_eq!(token.issuer, owner);
        // add token by not owner will fail
        let result = tool.add_token(&new_owner, token_info.clone());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ToolError::OnlyOwnerAllowCallIt);
    }

    // test get set token wasm
    #[test]
    fn test_get_set_token_wasm() {
        let mut tool = IssuanceTool::new();
        let owner =
            Principal::from_text("qupnt-ohzy3-npshw-oba2m-sttkq-tyawc-vufye-u5fbz-zb6yu-conr3-tqe")
                .unwrap();
        let result = tool.set_owner(&owner, owner.clone());
        assert!(result.is_ok());
        // set token wasm, check result is ok
        let token_wasm = vec![1, 2, 3, 4, 5];
        let result = tool.set_token_wasm(&owner, token_wasm.clone());
        // check result is ok
        assert!(result.is_ok());
        // get token wasm, check the wasm is equal token_wasm
        let token_wasm2 = tool.get_token_wasm().unwrap();
        assert_eq!(token_wasm, token_wasm2);
    }

    // test to payload / load from payload
    #[test]
    fn test_payload() {
        let mut tool = IssuanceTool::new();
        let owner =
            Principal::from_text("qupnt-ohzy3-npshw-oba2m-sttkq-tyawc-vufye-u5fbz-zb6yu-conr3-tqe")
                .unwrap();
        let token_id = Principal::from_text("g7cye-cyaaa-aaaak-aaa5a-cai").unwrap();
        let token_info = TokenInfo {
            issuer: owner.clone(),
            token_id: token_id.clone(),
            name: "test".to_string(),
            symbol: "TST".to_string(),
            decimals: 18,
            total_supply: 100,
            fee: Fee {
                minimum: Nat::from(1),
                rate: Nat::from(10000),
            },
            timestamp: 0,
        };
        let result = tool.set_owner(&owner, owner.clone());
        assert!(result.is_ok());
        let result = tool.add_token(&owner, token_info.clone());
        assert!(result.is_ok());
        let payload = tool.to_payload();
        tool.load_from_payload(payload.clone());
        let payload2 = tool.to_payload();
        // check payload is equal
        assert_eq!(payload, payload2);
    }
}
