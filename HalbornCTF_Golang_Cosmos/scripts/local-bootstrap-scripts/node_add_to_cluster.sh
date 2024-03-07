#!/bin/bash

set -e

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
source "${DIR}/lib/common.sh"

# Input check: node ID
if [ $# -eq 0 ]; then
  echo "Usage: node_add_to_cluster.sh [-c config_path] node_id"
  exit
fi

NODE_ID=$1
if [ "${NODE_ID}" -lt 0 ]; then
  echo "node_id must be GTE 0"
  exit
fi

# Define node params
NODE_DIR="${NODE_DIR_PREFIX}${NODE_ID}"
NODE_MONIKER="${NODE_MONIKER_PREFIX}${NODE_ID}"

NODE1_RPC_URL="http://127.0.0.1:${NODE_RPC_PORT_PREFIX}1"
NODE_RPC_URL="http://127.0.0.1:${NODE_RPC_PORT_PREFIX}${NODE_ID}"

CLI_COMMON_FLAGS="--home ${NODE_DIR}"

echo "-> Configuring node_${NODE_ID} (cluster should be up and running)"
  echo "Preparing directories"
    rm -rf "${NODE_DIR}"
    mkdir -p "${NODE_DIR}"
  echo

  echo "Init node: ${NODE_ID}"
    cli_init_flags="${CLI_COMMON_FLAGS} --chain-id ${CHAIN_ID}"

    # >>
    ${COSMOSD} init ${NODE_MONIKER} ${cli_init_flags} &> "${COMMON_DIR}/${NODE_MONIKER}_info.json"
    ${COSMOSD} set-genesis-defaults ${CLI_COMMON_FLAGS} > /dev/null

    AppConfig_setPorts ${NODE_ID}
  echo

  echo "Copy genesis"
    cp "${COMMON_DIR}/genesis.json" "${NODE_DIR}/config/genesis.json"
  echo

  echo "Configuring seed node (node_0)"
    seed_id_file_path="${COMMON_DIR}/node_0_nodeID"
    seed_id=$(cat ${seed_id_file_path})
    seed_p2p_port="${NODE_P2P_PORT_PREFIX}0"

    node_config_path="${NODE_DIR_PREFIX}${NODE_ID}/config/config.toml"
    seeds="${seed_id}@127.0.0.1:${seed_p2p_port}"
    sed -i.bak -e 's;seeds = \"\";seeds = \"'"${seeds}"'\";' "${node_config_path}"
  echo
echo "-> Done"
echo

echo "-> Starting node: tmux session: node_${NODE_ID}"
  session_id="node_${NODE_ID}"
  runner="${DIR}/node_run.sh ${NODE_ID}"
  tmux new -d -s ${session_id} ${runner}
echo

echo "-> Wait for chain init"
  cli_start_flags="${CLI_COMMON_FLAGS}"

  cur_block=0
  target_block=10
  while [ ${cur_block} -lt ${target_block} ]; do
    sleep 1

    # >>
    cur_block=$(${COSMOSD} q --node ${NODE_RPC_URL} block | jq '.block.header.height' | bc)
    re='^[0-9]+$'
    if ! [[ $cur_block =~ $re ]] ; then
        cur_block=0
    fi
    
    # >>
    target_block=$(${COSMOSD} q --node ${NODE1_RPC_URL} block | jq '.block.header.height' | bc)
    echo "CurrentBlock / TargetBlock: ${cur_block} / ${target_block}"
  done
echo "-> Done"
echo

echo "-> Create validator key"
  Keys_createSafe "${ACCPREFIX_VALIDATOR}${NODE_ID}"
echo "-> Done"
echo

echo "-> Send (self-stake + fee) coins from bank to validator acc"
  cli_bank_keys="--node ${NODE1_RPC_URL} --chain-id ${CHAIN_ID} --keyring-backend ${KEYRING_BACKEND} --keyring-dir ${KEYRING_DIR}"

  bank_addr=$(Keys_getAddr ${ACCNAME_BANK})
  validator_addr=$(Keys_getAddr "${ACCPREFIX_VALIDATOR}${NODE_ID}")

  # >>
  printf '%s\n%s\n' ${PASSPHRASE} ${PASSPHRASE} | ${COSMOSD} tx bank send ${bank_addr} ${validator_addr} ${NEWNODE_ACC_COINS} ${cli_bank_keys} --yes --broadcast-mode block
echo "-> Done"
echo

echo "-> Create validator with staking"
  cli_tm_flags="${CLI_COMMON_FLAGS}"
  cli_staking_flags="--node ${NODE1_RPC_URL} --chain-id ${CHAIN_ID} --keyring-backend ${KEYRING_BACKEND} --keyring-dir ${KEYRING_DIR}"

  # >>
  validator_pubkey=$(${COSMOSD} tendermint show-validator ${cli_tm_flags})
  printf '%s\n%s\n' ${PASSPHRASE} ${PASSPHRASE} | ${COSMOSD} tx staking create-validator ${cli_staking_flags} --amount=${NEWNODE_STAKE} --pubkey=${validator_pubkey} --moniker=${NODE_MONIKER} --commission-rate="0.10" --commission-max-rate="0.20" --commission-max-change-rate="0.01" --min-self-delegation="${MIN_SELF_DELEGATION_AMT}" --from "${validator_addr}" --yes --broadcast-mode block
echo "-> Done"
echo

echo "-> New validator node_${NODE_ID} with address ${validator_pubkey} created:"
  # >>
  ${COSMOSD} q staking delegations ${validator_addr} --node ${NODE1_RPC_URL}
