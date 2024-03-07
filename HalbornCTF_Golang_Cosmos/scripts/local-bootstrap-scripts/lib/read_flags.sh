# Working directory
COMMON_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

# Read common flags
CONFIG_PATH=""
ARGS_ALL=$@

args=()
while [[ $# -gt 0 ]]; do
  case $1 in
    ## Config path
    -c|--config)
      CONFIG_PATH="$2"
      shift
      shift
      ;;
    ## Other input argument
    *)
      args+=("$1")
      shift
      ;;
  esac
done
set -- "${args[@]}"

# Load config
if [ -z "${CONFIG_PATH}" ]; then
  echo "ERROR: config file path flag (-c|--config) not provided"
  exit 1
fi
source "${CONFIG_PATH}"
