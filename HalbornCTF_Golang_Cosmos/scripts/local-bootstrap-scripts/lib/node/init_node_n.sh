node_id=$1

# Define node params
node_dir="${NODE_DIR_PREFIX}${node_id}"
node_moniker="${NODE_MONIKER_PREFIX}${node_id}"
cli_common_flags="--home ${node_dir}"

##
echo "Preparing directories"
  rm -rf "${node_dir}"

  mkdir -p "${node_dir}"
echo

##
echo "Init node: ${node_id}"
  cli_init_flags="${cli_common_flags} --chain-id ${CHAIN_ID}"

  # >>
  ${COSMOSD} init ${node_moniker} ${cli_init_flags} &> "${COMMON_DIR}/${node_moniker}_info.json"

  AppConfig_setPorts ${node_id}

  echo "  PEX:                  off"
  echo "  Seed mode:            off"
  echo "  AddrBook strict mode: off"
  sed -i.bak -e 's;pex = true;pex = false;' "${node_dir}/config/config.toml"
  sed -i.bak -e 's;addr_book_strict = true;addr_book_strict = false;' "${node_dir}/config/config.toml"
echo

##
echo "Genesis TX to create validator with default min self-delegation and min self-stake"
  if ! $SKIP_GENESIS_OPS; then
    cli_gentx_flags="${cli_common_flags} --chain-id ${CHAIN_ID} --min-self-delegation ${MIN_SELF_DELEGATION_AMT} --keyring-backend ${KEYRING_BACKEND} --keyring-dir ${KEYRING_DIR} --output-document ${COMMON_DIR}/gentx/${node_id}_gentx.json"

    cp "${COMMON_DIR}/genesis.json" "${node_dir}/config/genesis.json"

    # >>
    printf '%s\n%s\n%s\n' ${PASSPHRASE} ${PASSPHRASE} ${PASSPHRASE} | ${COSMOSD} gentx ${ACCPREFIX_VALIDATOR}${node_id} ${BASENODE_STAKE} ${cli_gentx_flags}
  else
    echo "  Operation skipped"
  fi
echo

##
echo "Collect peers data"
  cli_tm_flags="${cli_common_flags}"

  # >>
  ${COSMOSD} tendermint show-node-id ${cli_tm_flags} > "${COMMON_DIR}/${node_moniker}_nodeID"
echo
