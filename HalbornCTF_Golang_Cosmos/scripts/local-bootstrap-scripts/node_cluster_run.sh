#!/bin/bash

set -e

# Imports
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
source "${DIR}/lib/read_flags.sh"
source "${DIR}/lib/node/common.sh"

for i in $(seq 0 $NODES_CNT); do
  session_id="${CHAIN_ID}_node_${i}"

  echo "-> Starting node: tmux session: ${session_id}"
  runner="${DIR}/node_run.sh ${ARGS_ALL} ${i}"
  tmux new -d -s ${session_id} ${runner}
done
