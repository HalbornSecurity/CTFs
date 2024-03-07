# Relayer home directory
RELAYER_DIR="${HOME}/HalbornSecurity/local/relayer_gaia12"

# Chain cluster configs
CHAIN1_CONFIG="config/gaia-1.sh"
CHAIN2_CONFIG="config/gaia-2.sh"

# Relayer default IBC timeout
TIMEOUT="20s"

# Paths (channels)
## This is a BASH array of arrays.
## Each array defines a single relayer path with the following values (by index):
##   0. "1->2" / "2->1". Defines a direction (chain 1 to chain 2 or vice versa);
##   1. Source port ID;
##   2. Destination port ID;
##   3. Order type ("ordered" / "unordered");
##   4. Version;
declare -a transfer_path12=("1->2" "transfer" "transfer" "unordered" "ics20-1")
declare -a PATHS=("transfer_path12")
