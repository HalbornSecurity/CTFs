#!/bin/bash

set -e

# Imports
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
source "${DIR}/lib/read_flags.sh"
source "${DIR}/lib/utils.sh"

# tmux
sessions=($(tmux ls -F "#{session_name}"))
for session in "${sessions[@]}"; do
  terminate=false
	if [[ $session == ${CHAIN_ID}_node_* ]]; then terminate=true; fi

	if [ "$terminate" = true ]; then
	  echo "-> Stopping ${session} tmux session"
	  Kill_tmux_session "${session}"
	fi
done

# Docker
# dvm_containers=$(docker ps --filter "name=dvm_*" --filter "status=running" --filter "status=exited" -q)
# [ ! -z "${dvm_containers}" ] && echo "-> Stopping DVM containers" && docker rm -f ${dvm_containers}
