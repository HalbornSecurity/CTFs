#!/bin/bash

set -e

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

RELAYER_CONFIG="$1"
CHAIN_CONFIG="$2"

source "${RELAYER_CONFIG}"
source "$(dirname ${DIR})/read_flags.sh" -c "${CHAIN_CONFIG}"
source "$(dirname ${DIR})/node/common.sh"

#
echo "Relayer chain config build and upload"
  chain_relayer_config="${RELAYER_DIR}/${CHAIN_ID}_chain.json"

  cp "${DIR}/chain.json.template" "${chain_relayer_config}"
  sed -i.bak -e 's;{relayerAccount};'"${CHAIN_ID}_relayer"';' "${chain_relayer_config}"
  sed -i.bak -e 's;{chainID};'"${CHAIN_ID}"';' "${chain_relayer_config}"
  sed -i.bak -e 's;{bech32Prefix};'"${BECH32_PREFIX}"';' "${chain_relayer_config}"
  sed -i.bak -e 's;{rpcPort};'"${NODE_RPC_PORT_PREFIX}1"';' "${chain_relayer_config}"
  sed -i.bak -e 's;{keyringBackend};'"${KEYRING_BACKEND}"';' "${chain_relayer_config}"
  sed -i.bak -e 's;{gasDenom};'"${STAKE_DENOM}"';' "${chain_relayer_config}"
  sed -i.bak -e 's;{timeout};'"${TIMEOUT}"';' "${chain_relayer_config}"
  rm "${chain_relayer_config}.bak"

  # >>
  rly --home "${RELAYER_DIR}" chains add -f ${chain_relayer_config}
  
  echo "  Chain config: ${chain_relayer_config}"
echo

#
echo "Restoring relayer account key"
  relayer_accname="${CHAIN_ID}_relayer"
  mnemonic="$(Keys_getMnemonic ${relayer_accname})"

  # >>
  set +e
  rly --home "${RELAYER_DIR}" keys delete ${CHAIN_ID} ${relayer_accname} -y 2>&1
  rly --home "${RELAYER_DIR}" keys restore ${CHAIN_ID} ${relayer_accname} "${mnemonic}" 2>&1
  set -e

  echo "  Account key added: ${relayer_accname}"
echo
