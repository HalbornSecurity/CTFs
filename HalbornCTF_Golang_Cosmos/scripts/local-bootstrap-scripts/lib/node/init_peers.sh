echo "Reading node IDs"
  i=0
  while read file; do
    file_path="${COMMON_DIR}/${file}"
    node_id=$(cat ${file_path})
    
    node_ids[$i]="${node_id}"
    (( i++ ))
  done < <(ls ${COMMON_DIR} | grep nodeID)
echo

echo "Updating seeds for nodes"
  seeds=""

  for i in ${!node_ids[@]}; do
    if [ $i -eq 0 ]; then
      seeds="${node_ids[$i]}@127.0.0.1:${NODE_P2P_PORT_PREFIX}0"
      continue
    fi

    config_path="${NODE_DIR_PREFIX}${i}/config/config.toml"
    sed -i.bak -e 's;seeds = \"\";seeds = \"'"${seeds}"'\";' "${config_path}"

    echo "  Node [${i}] seeds (${config_path}): ${seeds}"
  done
echo

echo "Fix invalid default config.toml -> persistent_peers filed for node_0"
  config_path="${NODE_DIR_PREFIX}0/config/config.toml"
  old_peers_line=$(sed -n -e '/^persistent_peers = /p' "${config_path}")

  new_peers=()
  for i in ${!node_ids[@]}; do
    if [ "$i" -eq 0 ]; then continue; fi

    node_id="${node_ids[$i]}"
    node_p2p_port="${NODE_P2P_PORT_PREFIX}${i}"
    new_peers+=( "${node_id}@127.0.0.1:${node_p2p_port}" )
  done

  new_peers_line="persistent_peers = \"$(ArrayJoin , "${new_peers[@]}\"")"
  echo "  Old peers line: ${old_peers_line}"
  echo "  New peers line: ${new_peers_line}"
  sed -i.bak -e "s;${old_peers_line};${new_peers_line};" "${config_path}"
echo
