#!/bin/bash

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
source "${DIR}/lib/common.sh"

# Input checks
if [ $# -eq 0 ]; then
  echo "Usage: import_genesis_acc.sh [-c config_path] name index mnemonic [account_number]"
  exit
fi

NAME=$1
if [ -z "${NAME}" ]; then
  echo "name: empty"
  exit
fi

INDEX=$2
if [ -z "${INDEX}" ]; then
  echo "index: empty"
  exit
fi

MNEMONIC=$3
if [ -z "${MNEMONIC}" ]; then
  echo "mnemonic: empty"
  exit
fi

NUMBER=0
if [ ! -z "$4" ]; then
  NUMBER=$4
fi

echo "-> Importing account"
  echo "  Name:   ${NAME}"
  echo "  Index:  ${INDEX}"
  echo "  Number: ${NUMBER}"

  cli_keys_flags="--keyring-backend ${KEYRING_BACKEND} --keyring-dir ${KEYRING_DIR} --output json"

  # >>
  ${COSMOSD} keys delete -y ${NAME} ${cli_keys_flags} > /dev/null 2>&1
  echo "${COSMOSD} keys add --recover --account ${NUMBER} --index ${INDEX} ${NAME} ${cli_keys_flags}"
  acc_data=`echo ${MNEMONIC} | ${COSMOSD} keys add --recover --account ${NUMBER} --index ${INDEX} ${NAME} ${cli_keys_flags} 2>&1`

  echo ${acc_data} > "${KEYRING_DIR}/${NAME}_key"
  acc_data=$(echo ${acc_data} | jq ".mnemonic = \"${MNEMONIC}\"")
  echo ${acc_data}
  echo ${acc_data} > "${KEYRING_DIR}/${NAME}_key"
echo "-> Done"
