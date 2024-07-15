#!/usr/bin/env bash

set -e
set -o pipefail

workspace_version="0.16.0"
cw20_version="v2.0.0"

projectPath=$(cd "$(dirname "${0}")" && cd ../ && pwd)
artifactPath="$projectPath/artifacts"
cw20file="$artifactPath/cw20_base.wasm"

docker run --rm \
    --volume "$projectPath":/code \
    --volume "$(basename "$projectPath")-target":/code/target \
    --volume cargo-registry:/usr/local/cargo/registry \
    cosmwasm/workspace-optimizer:$workspace_version

sudo wget -q -O $cw20file  "https://github.com/CosmWasm/cw-plus/releases/download/$cw20_version/cw20_base.wasm"