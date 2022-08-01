# Chain IDs from configs
CHAIN1_ID=$(sed -n 's/^CHAIN_ID="\(.*\)"/\1/p' "${CHAIN1_CONFIG}")
CHAIN2_ID=$(sed -n 's/^CHAIN_ID="\(.*\)"/\1/p' "${CHAIN2_CONFIG}")
