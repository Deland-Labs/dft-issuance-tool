#!/usr/bin/ic-repl
import tool = "rrkah-fqaaa-aaaaa-aaaaq-cai" as ".dfx/local/canisters/issuanceTool/issuanceTool.did";
identity default "~/.config/dfx/identity/default/identity.pem";
call tool.uploadTokenWasm(
  record {
    wasm_module = file "wasm/dft_rs_opt.wasm";
  },
);
let result = _ ;
call tool.uploadTokenStorageWasm(
  record {
    wasm_module = file "wasm/graphql_opt.wasm";
  },
);

assert _ == result