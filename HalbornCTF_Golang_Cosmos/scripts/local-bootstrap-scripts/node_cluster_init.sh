#!/bin/bash

set -e

# Imports
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
source "${DIR}/lib/read_flags.sh"
source "${DIR}/lib/utils.sh"
source "${DIR}/lib/node/common.sh"

echo "-> Configuring node 0 (seed)"
  source "${DIR}/lib/node/init_node_0.sh"
echo "-> Done"
echo

for i in $(seq 1 $NODES_CNT); do
  echo "-> Configuring node ${i}"
  source "${DIR}/lib/node/init_node_n.sh" ${i}
  echo "-> Done"
  echo
done

echo "-> Configuring genesis"
  source "${DIR}/lib/node/init_genesis.sh"
echo "-> Done"

echo "-> Configuring p2p network"
  source "${DIR}/lib/node/init_peers.sh"
echo "-> Done"
echo
