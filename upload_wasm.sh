#!/usr/bin/ic-repl
import tool = "rrkah-fqaaa-aaaaa-aaaaq-cai" as ".dfx/local/canisters/issuanceTool/issuanceTool.did";
identity default "~/.config/dfx/identity/dft_tool/identity.pem";
call tool.uploadTokenWasm(
  record {
    wasm_module = file "wasm/dft_basic_opt.wasm";
  },
);