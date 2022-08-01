IFS=',' read -r -a SKIP_GENACC_NAMES <<< "${SKIP_GENACC_NAMES}"

# Define node params
node_id=0
node_dir="${NODE_DIR_PREFIX}${node_id}"
node_moniker="${NODE_MONIKER_PREFIX}${node_id}"
cli_common_flags="--home ${node_dir}"

##
echo "Preparing directories"
  rm -rf "${COMMON_DIR}"
  rm -rf "${node_dir}"

  mkdir -p "${KEYRING_DIR}"
  mkdir -p "${COMMON_DIR}/gentx"
  mkdir -p "${node_dir}"
echo

##
echo "Init node: 0"
  cli_init_flags="${cli_common_flags} --chain-id ${CHAIN_ID}"

  # >>
  ${COSMOSD} init ${node_moniker} ${cli_init_flags} &> "${COMMON_DIR}/${node_moniker}_info.json"

  AppConfig_setPorts ${node_id}

  echo "  PEX:                  on"
  echo "  Seed mode:            on"
  echo "  AddrBook strict mode: on"
  sed -i.bak -e 's;seed_mode = false;seed_mode = true;' "${node_dir}/config/config.toml"
  sed -i.bak -e 's;addr_book_strict = true;addr_book_strict = false;' "${node_dir}/config/config.toml"

  if [ ! -z "${EXPORTED_GENESIS}" ]; then
    echo "  Replace default genesis with an exported one"
    cp "${EXPORTED_GENESIS}" "${node_dir}/config/genesis.json"
  fi
echo

##
echo "Fix for single node setup"
  NODES_CNT_FIX=$NODES_CNT
  if [ "${NODES_CNT}" -eq "1" ]; then
    NODES_CNT_FIX=3
    echo "  Hard fix for 3 nodes"
  fi
echo

##
echo "Build account names list (keys add, add-genesis-account)"
  accnames_unfiltered=("${ACCNAME_BANK}")
  accnames_unfiltered=("${accnames_unfiltered[@]}" "${EXTRA_ACCOUNTS[@]} ")
  for i in $(seq 1 $NODES_CNT_FIX); do
    accnames_unfiltered+=("${ACCPREFIX_VALIDATOR}${i}")
  done

  accnames=()
  for accname_raw in "${accnames_unfiltered[@]}"; do
    skip=false

    for accname_filtered in "${SKIP_GENACC_NAMES[@]}"; do
      if [ "${accname_raw}" == "${accname_filtered}" ]; then
        skip=true
        echo "  ${accname_raw}: skipped"
        break
      fi
    done

    if ! $skip; then
      accnames+=(${accname_raw})
    fi
  done

  echo "  Active account names: ${accnames[@]}"
echo

##
echo "Add keys"
  for accname in "${accnames[@]}"; do
    Keys_createSafe ${accname}
    # Keys_createWithOverride ${accname}
    echo "  ${accname}: key created (or skipped if already exists)"
  done
echo

##
echo "Add genesis accounts"
  if ! $SKIP_GENESIS_OPS; then
    cli_genacc_flags="${cli_common_flags} --keyring-backend ${KEYRING_BACKEND} --output json"

    for accname in "${accnames[@]}"; do
      # >>
      ${COSMOSD} add-genesis-account $(Keys_getAddr ${accname}) ${GENACC_COINS} ${cli_genacc_flags}

      echo "  ${accname}: genesis account added"
    done
  else
    echo "  Operation skipped"
  fi
echo

##
echo "Change other genesis settings"
  if ! $SKIP_GENESIS_OPS; then
    echo "  Changing Gov voting period to 300s"
      jq '.app_state.gov.voting_params.voting_period = "300s"' "${node_dir}/config/genesis.json" > "${node_dir}/config/tmp.json"
      mv "${node_dir}/config/tmp.json" "${node_dir}/config/genesis.json"
    echo

    echo "  Adding x/hal metadata for supported collaterals [usdt, usdc, busd]"
      usdc_meta='{ "denom": "musdc", "decimals": 3, "description": "USDC native token (milli USDC)" }'
      usdt_meta='{ "denom": "uusdt", "decimals": 6, "description": "USDT native token (micro USDT)" }'
      busd_meta='{ "denom": "nbusd", "decimals": 9, "description": "BUSD native token (nano BUSD)" }'

      jq --argjson usdt "${usdt_meta}" --argjson usdc "${usdc_meta}" --argjson busd "${busd_meta}" '.app_state.hal.params.collateral_metas += [ $usdt, $usdc, $busd ]' "${node_dir}/config/genesis.json" > "${node_dir}/config/tmp.json"
      mv "${node_dir}/config/tmp.json" "${node_dir}/config/genesis.json"
    echo

    echo "  Changing HAL redeem period to 15s"
      jq '.app_state.hal.params.redeem_dur = "15s"' "${node_dir}/config/genesis.json" > "${node_dir}/config/tmp.json"
      mv "${node_dir}/config/tmp.json" "${node_dir}/config/genesis.json"
    echo
  else
    echo "  Operation skipped"
  fi
echo

##
echo "Validate genesis"
  # >>
  ${COSMOSD} validate-genesis "${node_dir}/config/genesis.json" ${cli_common_flags}

  cp "${node_dir}/config/genesis.json" "${COMMON_DIR}/genesis.json"
echo

##
echo "Collect peers data"
  cli_tm_flags="${cli_common_flags}"

  # >>
  ${COSMOSD} tendermint show-node-id ${cli_tm_flags} > "${COMMON_DIR}/${node_moniker}_nodeID"
echo
