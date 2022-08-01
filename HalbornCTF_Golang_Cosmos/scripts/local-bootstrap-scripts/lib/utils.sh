# Kill tmux session by {session_id}
function Kill_tmux_session() {
  session=$1

  for pane in `tmux list-panes -F '#{pane_id}' -t ${session}`; do
    # send SIGINT to all panes in selected window
    tmux send-keys -t $pane C-c
    echo ${session}:$name.${pane//%}
  done

  for pane in `tmux list-panes -F '#{pane_pid}' -t ${session}`; do
    # terminate pane
    kill -TERM ${pane}
  done
}

# Join string array [$2...] with $1 IFS delimiter
function ArrayJoin {
  local IFS="$1"; shift; echo "$*";
}
