# Binaries
## Cosmos-based chain binary path.
COSMOSD="gaiad"

# Chain ID
## That ID is also used as an account name prefix for default accounts ({CHAIN_ID}_local-bank, {CHAIN_ID}_local-validator-1, ...)
CHAIN_ID="gaia-1"

# Account address prefix
BECH32_PREFIX="cosmos"

# Node port prefixes
NODE_P2P_PORT_PREFIX="2666"
NODE_RPC_PORT_PREFIX="2667"
NODE_PROXY_PORT_PREFIX="2665"
NODE_GRPC_PORT_PREFIX="919"
NODE_GRPC_WEB_PORT_PREFIX="929"

# Local cluster file path base
## Account secrets, nodes data and configs, genesis, genTxs, etc.
CLUSTER_DIR="${HOME}/USC/local/${CHAIN_ID}"

# Cluster size (must be GTE 1)
## Nodes can be added later using node_add_to_cluster.sh script.
NODES_CNT=3

# Exported genesis path
## If not empty, the default generated genesis file is replaced with that one.
EXPORTED_GENESIS=""

# Skip genesis generation operations
## Oracle and assets creation, genTxs, etc.
SKIP_GENESIS_OPS=false

# List of account names that should not be generates ("local-validator-1,local-validator-2,local-validator-3,local-oracle1")
## Those accounts should be imported to the local keyring using import_genesis_acc.sh script before cluster init.
SKIP_GENACC_NAMES=""

# Main coin denom for staking and fees
STAKE_DENOM="stake"

# Generated accounts balance (comma-separated list)
GENACC_COINS="1000000000${STAKE_DENOM},100000musdc,100000000uusdt,100000000000nbusd"

# Additional accounts to be created with ${GENACC_COINS} balances (besides "standard" bank, validators) [BASH array]
EXTRA_ACCOUNTS=("${CHAIN_ID}_relayer" "${CHAIN_ID}_a" "${CHAIN_ID}_b")

# Min self delegation amount for all validators (base and new)
MIN_SELF_DELEGATION_AMT="1000000000"

# Self-delegation value for base validators
BASENODE_STAKE="${MIN_SELF_DELEGATION_AMT}${STAKE_DENOM}"

# New validator balance that is transferred from local-bank
## Value is used by the node_add_to_cluster.sh script to transfer some tokens for a new validator to start.
NEWNODE_ACC_COINS="1000000000${STAKE_DENOM}"

# Self-delegation value for new validators
NEWNODE_STAKE="${MIN_SELF_DELEGATION_AMT}${STAKE_DENOM}"

# Node logging level
NODE_LOGLEVEL="info"

# Kering storage
## "os" / "file".
KEYRING_BACKEND="os"
