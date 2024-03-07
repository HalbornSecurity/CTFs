#!/bin/bash

set -e

# Imports
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
source "${DIR}/lib/read_flags.sh"

# Input check: target block
if [ $# -eq 0 ]; then
  echo "Usage: wait_for_block.sh -c config_path targetBlockNumber"
  exit
fi
TARGET_BLOCK=$1

blockGetter="${COSMOSD} q --node tcp://localhost:${NODE_RPC_PORT_PREFIX}1 block | jq -r \".block.header.height\""

echo "-> Waiting for block ${TARGET_BLOCK}"
while : ; do
  block=$(eval ${blockGetter})
  echo "  Block: ${block} / ${TARGET_BLOCK}"
  if [ "${block}" -ge "${TARGET_BLOCK}" ]; then
    break
  fi
  sleep 5
done
echo "-> Done"
echo
