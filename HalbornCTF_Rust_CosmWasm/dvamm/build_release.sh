# Optimized builds
docker run --rm -v "$(pwd)":/code --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry cosmwasm/workspace-optimizer:0.12.3

set -e

projectPath=$(cd "$(dirname "${0}")"  && pwd)
artifactPath="$projectPath/artifacts"
cw20file="$artifactPath/cw20_base.wasm"

wget -q -O $cw20file  'https://github.com/CosmWasm/cw-plus/releases/download/v0.8.1/cw20_base.wasm'