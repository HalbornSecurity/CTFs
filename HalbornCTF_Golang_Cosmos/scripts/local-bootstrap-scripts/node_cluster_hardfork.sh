#!/bin/bash

set -e

# Imports
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
source "${DIR}/lib/read_flags.sh"
source "${DIR}/lib/utils.sh"
source "${DIR}/lib/node/common.sh"

GEN_EXPORT_PATH="${COMMON_DIR}/hardfork_genesis.json"
NODE1_HOME="${NODE_DIR_PREFIX}1"
NODE1_RPC_URL="http://127.0.0.1:${NODE_RPC_PORT_PREFIX}1"

echo "-> Building base nodes white list"
  base_node_addrs=()
  for i in $(seq 1 $NODES_CNT); do
    # >>
    val_op_addr=$(${COSMOSD} q --node "${NODE1_RPC_URL}" --output json staking validators | jq -r ".validators[] | select(.description.moniker == \"${NODE_MONIKER_PREFIX}${i}\") | .operator_address")

    base_node_addrs+=(${val_op_addr})
    echo "  Base node [${i}]: ${val_op_addr}"
  done

  WHITE_LIST=$(ArrayJoin , ${base_node_addrs[@]})
  echo "  ValOperators list to skip jailing: ${WHITE_LIST}"
echo "-> Done"
echo

echo "-> Stop all"
  "${DIR}/stop_all.sh"
echo "-> Done"
echo

echo "-> Genesis export: ${GEN_EXPORT_PATH}"
  # >>
  ${COSMOSD} --home "${NODE1_HOME}" export --for-zero-height --jail-allowed-addrs "${WHITE_LIST}" > "${GEN_EXPORT_PATH}"
echo "-> Done"
echo

echo "-> Reset nodes data (unsafe-reset-all)"
  for file in ${CLUSTER_DIR}/*/; do
    if [ ! -d "$file" ]; then
      continue
    fi

    if [[ $file != ${NODE_DIR_PREFIX}* ]]; then
      continue
    fi

    node_id=${file%*/}
    node_id=${node_id#"${NODE_DIR_PREFIX}"}

    # >>
    ${COSMOSD} --home "$file" unsafe-reset-all
    
    echo "  Node [${node_id}]: reset"

    cp "${GEN_EXPORT_PATH}" "${file}/config/genesis.json"
    echo "  Node [${node_id}]: genesis file replaced with the exported one"
  done
echo "-> Done"
echo
