#!/usr/bin/env bash

set -e

projectPath=$(cd "$(dirname "${0}")" && cd ../ && pwd)

artifactPath="$projectPath/artifacts"
if [ ! -d "$artifactPath" ]; then
    npm run build-artifacts
fi

terraLocalPath="${TERRA_LOCAL_PATH:-"$(dirname "$projectPath")/terra-local"}"
if [ ! -d "$terraLocalPath" ]; then
    git clone --depth 1 https://www.github.com/terra-money/LocalTerra "$terraLocalPath"
fi

docker-compose --project-directory "$terraLocalPath" rm --force --stop && docker-compose --project-directory "$terraLocalPath" up --detach

sleep 5 # waite startup terra local

rm -fr "$projectPath/artifacts/localterra.json"
