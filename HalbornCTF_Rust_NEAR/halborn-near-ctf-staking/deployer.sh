#!/bin/bash

rm -rf neardev
cargo clean
cargo build --target wasm32-unknown-unknown --release
ACCOUNT=$(near dev-deploy --wasmFile target/wasm32-unknown-unknown/release/halborn_near_ctf_staking.wasm | sed -n '5,1p' | cut -d " " -f 4)
echo Staking contract deployed to $ACCOUNT