#!/bin/bash

set -e

# Imports
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
source "${DIR}/lib/read_flags.sh"
source "${DIR}/lib/node/common.sh"

# Input check: nodeID
if [ $# -eq 0 ]; then
  echo "Usage: node_run.sh -c config_path node_id"
  exit
fi

node_id=$1
if [ "${node_id}" -lt 0 ]; then
  echo "node_id: must be GTE 0"
  exit
fi
node_dir="${NODE_DIR_PREFIX}${node_id}"

echo "-> Starting node ${node_id}"
  echo "  Node dir: ${node_dir}"

  # >>
  ${COSMOSD} start --home="${node_dir}" --inv-check-period=2 --log_level=${NODE_LOGLEVEL}
echo "-> Node stopped"
