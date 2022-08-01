#!/bin/bash

set -e

# Imports
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
source "${DIR}/lib/read_flags.sh"
source "${DIR}/lib/relayer/common.sh"

# Start serving
debug_port_prefix="localhost:759"
idx=0
for path_name in "${PATHS[@]}"; do
  ((idx=idx+1))
  session_id="${CHAIN1_ID}_${CHAIN2_ID}_relay_${path_name}"

  echo "-> Starting relayer: tmux session: ${session_id}"
  runner="rly --home ${RELAYER_DIR} start ${path_name} --debug-addr ${debug_port_prefix}${idx}"
  tmux new -d -s ${session_id} ${runner}
  echo
done
