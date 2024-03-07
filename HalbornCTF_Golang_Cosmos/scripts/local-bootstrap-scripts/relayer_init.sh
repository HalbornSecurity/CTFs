#!/bin/bash

set -e

# Imports
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
source "${DIR}/lib/read_flags.sh"
source "${DIR}/lib/relayer/common.sh"

#
echo "-> Preparing directories"
  rm -rf "${RELAYER_DIR}"
  mkdir -p "${RELAYER_DIR}"

  # >>
  rly config init --home "${RELAYER_DIR}"
echo "-> Done"
echo

#
echo "-> Chain 1 configuration"
  ${DIR}/lib/relayer/init_chain.sh "${CONFIG_PATH}" "${CHAIN1_CONFIG}"
echo "-> Done"
echo

echo "-> Chain 2 configuration"
  ${DIR}/lib/relayer/init_chain.sh "${CONFIG_PATH}" "${CHAIN2_CONFIG}"
echo "-> Done"
echo

#
echo "-> Channels configuration"
  for path_name in "${PATHS[@]}"; do
    path_data="$path_name[@]"
    
    path_opts=()
    path_opts+=("$path_name")
    for v in "${!path_data}"; do
      case $v in
        "1->2")
          path_opts+=("$CHAIN1_ID")
          path_opts+=("$CHAIN2_ID")
          ;;
        "2->1")
          path_opts+=("$CHAIN2_ID")
          path_opts+=("$CHAIN1_ID")
          ;;
       *)
         path_opts+=("$v")
         ;;
      esac
    done

    source "${DIR}/lib/relayer/init_path.sh" ${path_opts[@]}
  done
echo "-> Done"
echo

#
echo "-> Printing created channels status"
  for path_name in "${PATHS[@]}"; do
    # >>
    rly --home ${RELAYER_DIR} paths show ${path_name}
    echo
  done
echo "-> Done"
echo
