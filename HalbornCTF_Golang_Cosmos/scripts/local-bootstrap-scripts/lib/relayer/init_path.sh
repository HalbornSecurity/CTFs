path_name="$1"
chain_src="$2"
chain_dst="$3"
path_src_port="$4"
path_dst_port="$5"
path_order="$6"
path_version="$7"

echo "-> Creating path (${path_name}): ${chain_src}:${path_src_port} -> ${chain_dst}:${path_dst_port} (${path_order} / ${path_version})"
  # >>
  rly --home "${RELAYER_DIR}" paths new ${chain_src} ${chain_dst} ${path_name}
  rly --home "${RELAYER_DIR}" tx connect ${path_name} --src-port ${path_src_port} --dst-port ${path_dst_port} --order ${path_order} --version ${path_version}
echo
