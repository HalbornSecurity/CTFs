#!/bin/bash

rm -rf ./neardev

CTF_ASSOCIATED_CONTRACT=$(near dev-deploy --wasmFile target/wasm32-unknown-unknown/release/halborn_near_ctf_associated_contract.wasm --initFunction new --initArgs '{"owner_id": "TODO"}' | sed -n '5,1p' | grep -o -E "dev-\d+-\d+")

echo "CTF contract deployed to: $CTF_ASSOCIATED_CONTRACT"
