#!/bin/bash

set -e

# Imports
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
source "${DIR}/lib/read_flags.sh"
source "${DIR}/lib/utils.sh"
source "${DIR}/lib/relayer/common.sh"

# tmux
sessions=($(tmux ls -F "#{session_name}"))
for session in "${sessions[@]}"; do
  terminate=false
	if [[ $session == ${CHAIN1_ID}_${CHAIN2_ID}_relay_* ]]; then terminate=true; fi

	if [ "$terminate" = true ]; then
	  echo "-> Stopping ${session} tmux session"
	  Kill_tmux_session "${session}"
	fi
done
