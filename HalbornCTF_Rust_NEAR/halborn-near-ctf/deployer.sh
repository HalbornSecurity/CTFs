#!/bin/bash

rm -rf ./neardev

CTF_CONTRACT=$(near dev-deploy --wasmFile target/wasm32-unknown-unknown/release/halborn_near_ctf.wasm --initFunction new --initArgs '{"owner_id": "TODO", "token_total_supply": "1000000000000000000"}' | sed -n '5,1p' | grep -o -E "dev-\d+-\d+")

echo "CTF contract deployed to: $CTF_CONTRACT"
