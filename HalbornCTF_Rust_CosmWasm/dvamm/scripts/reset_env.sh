#!/usr/bin/env bash

# Script to reset localterra stored values

set -e

projectPath=$(cd "$(dirname "${0}")" && cd ../ && pwd)

artifactPath="$projectPath/artifacts"

terraLocalPath="${TERRA_LOCAL_PATH:-"$(dirname "$projectPath")/terra-local"}"

docker-compose --project-directory "$terraLocalPath"  rm -f -s -v
docker volume rm localterra_terra

rm -fr "$projectPath/artifacts/localterra.json"
