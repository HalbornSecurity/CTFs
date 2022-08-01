node_id=0
node_dir="${NODE_DIR_PREFIX}${node_id}"
cli_common_flags="--home ${node_dir}"

echo "Collect genesis TXs and validate"
  cp "${node_dir}/config/genesis.json" "${COMMON_DIR}/genesis.json.orig"

  # >>
  ${COSMOSD} collect-gentxs --gentx-dir "${COMMON_DIR}/gentx" ${cli_common_flags} &> /dev/null
  ${COSMOSD} validate-genesis ${cli_common_flags}
  
  cp "${node_dir}/config/genesis.json" "${COMMON_DIR}/genesis.json"
echo

echo "Distribute genesis.json to nodes"
  for i in $(seq 1 $NODES_CNT); do
    cp "${COMMON_DIR}/genesis.json" "${NODE_DIR_PREFIX}${i}/config/genesis.json"
    echo "  copied to node ${i}"
  done
echo
