# Local Cosmos chain cluster bootstrap scripts

Only for debug purposes.

This document placeholders:
* `{COSMOSD}` - defines a chain binary (`gaiad` for example);
* `{CLUSTER_DIR}` - local path to a directory which contains all nodes "homes", configs, genTXs, account details;

## Requirements

Different scripts require certain binaries to be build / downloaded / installed and available through `$PATH`.

Common requirements:
* `tmux` - used to run multiple processes within a single terminal session;
  * `brew install tmux` for OS X
* `jq` - used to alter JSON configs;
  * `brew install jq` for OS X
* `docker` - no comments;
* [Cosmos Relayer](https://github.com/cosmos/relayer) for IBC relayer bootstrap;

Some scripts do require additional apps to be available, refer to section's **Requirements** to find them.

## tmux cheatsheet

tmux is a cool tool similar to GNU Screen.
Once attached to a tmux session it is like you're using Vim. 

Outside of the `tmux` session:

    # List all active sessions
    tmux ls
    
    # Attach to an existing session
    tmux a -t {session_name}
    
    # Kill all tmux sessions and Docker container (only those started by this scripts)
    ./stop_all.sh

Inside of a `tmux` session:

    # Help
    Ctrl + b -> ?
    
    # Detach from the current session
    Ctrl + b -> d
    
    # Switch to prev session
    Ctrl + b -> (
    
    # Switch to next session
    Ctrl + b -> )
    
    # Search (like Vim)
    / [text] Enter
    
    # Terminate the current process
    Ctrl + c

Tmux has its own buffer and scrolling, so mouse scroll won't work by default.
To enable it edit the tmux config:

    vim ~/.tmux.conf
    
    # Add the line
    set -g mouse on

Some notes:
  * `tmux kill-session -t node_1` - that kills the session, but the process will keep running;
  * To kill the session you have to attach to it and terminate it manually (or use `stop_all.sh` script);

## Nodes cluster

Scripts to init and start Cosmos-based chain cluster to play with.
If `path_to_exported_genesis` argument is provided, cluster will start of from an exported genesis state (for migration tests).

### Requirements

Prepare cluster config file for your chain ([example](./config/arch.sh)). Since there could be multiple chains runnings at the same time, each script requires a path to the corresponding config.

### Scripts

* `node_cluster_init.sh -c path_to_config`
  * Example: `./node_cluster_init.sh -c config/gaiaA.sh`
  * Initializes genesis, configs and P2P network configuration for cluster of base nodes.
* `node_run.sh -c path_to_config node_ID`
  * Example: `./node_run.sh -c config/gaiaA.sh 1`
  * Starts the node within the current terminal session.
* `node_cluster_run.sh -c path_to_config`
  * Example: `./node_cluster_run.sh -c config/gaiaA.sh`
  * Runs the `node_run.sh` scripts.
  * Each node is `tmux`-ed into `{CHAIN_ID}_node_{NODE_ID}` tmux session.
* `node_add_to_cluster.sh -c path_to_config node_ID`
  * Example: `./node_add_to_cluster.sh -c config/gaiaA.sh 4`
  * Having a cluster of nodes up and running, script initializes a new one, wait for it to sync up and registers a new validator.
  * Script doesn't start the new node, use `node_run.sh` after this one.
* `import_genesis_acc.sh -c path_to_config acc_name acc_index acc_mnemonic [acc_number]`
  * Example: `./import_genesis_acc.sh -c config/gaiaA.sh local-bank 1 'secret'`
  * Script is useful when custom genesis file is used, and some account should be reused rather than generating it from scratch (`local-validator-1` for example).
* `wait_for_block.sh -c path_to_config target_block`
  * Example: `./wait_for_block.sh -c config/gaiaA.sh 500`
  * Script queries the 1st node and waits for it to reach the `{target_block}`.
* `stop_cluster.sh -c path_to_config`
  * Example: `./stop_cluster.sh -c config/gaiaA.sh`
  * Stops all tmux sessions for a chain.


### How does it work

All the keys, configs and data for each node instance are preserved within the folder `${HOME}/${CLUSTER_DIR}/local`.

On `{COSMOSD} start` `--home` argument provides home-directory for node instance. That way it is possible to run multiple node processes.

`{COSMOSD}` client (`tx`/`query` commands) also has a `--node` argument which defines node URL to connect to.

#### Keyring

The config variable `KEYRING_BACKEND` defines which secret storage to use:
* `os`
  * OS storage (for OSX: `Keychain access`);
  * Doesn't require the passphrase enter for each keyring operation;
* `file`
  * File based storage: `${HOME}/{CLUSTER_DIR}/local/keyring`;
  * Requires passphrase to be entered for each keyring operation;
  * Default passphrase: `passphrase`;

Script creates a new account key (`{CHAIN_ID}_local-bank`, `{CHAIN_ID}_local-validator1`, etc.) only if it is not present.
That way mnemonic and address is only created once per account.
Account  prefix is needed to avoid collisions with existing genesis account when an exported genesis is used or with other chain accounts.

Mnemonic and private keys can be found in the `${HOME}/{CLUSTER_DIR}/local/keyring` directory (`validator1_key`, `bank_key`, etc.).
Those files are created as a new account is created.

#### Aliases

To reduce the number of CLI arguments you might use BASH aliases (`~/.bashrc` / `~/.zshrc`):
* `alias c{COSMOSD}_1="{COSMOSD} --home $HOME/{CLUSTER_DIR}/local/node_1"`
* `alias c{COSMOSD}_2="{COSMOSD} --home $HOME/{CLUSTER_DIR}/local/node_2"`
* `alias c{COSMOSD}_3="{COSMOSD} --home $HOME/{CLUSTER_DIR}/local/node_3"`
* `alias c{COSMOSD}_4="{COSMOSD} --home $HOME/{CLUSTER_DIR}/local/node_4"`
* `alias c{COSMOSD}_q="{COSMOSD} q --node tcp://localhost:26671"`
* `alias c{COSMOSD}_tx="{COSMOSD} tx --chain-id {CHAIN_ID} --node tcp://localhost:26671 --keyring-backend os`
* `alias c{COSMOSD}_keys="{COSMOSD} keys --keyring-backend os`

Notes:
* `c{COSMOSD}_q` - used for Query requests;
* `c{COSMOSD}_tx` - used tor Tx requests;
* `c{COSMOSD}_keys` - used for keyring operations;
* If keyring-backend is set to `file`, `--keyring-dir $HOME/{CLUSTER_DIR}/local/common/keyring"` argument should be added to `c{COSMOSD}_tx` and `c{COSMOSD}_keys` aliases;

Examples:

    cgaiad_q bank balances $(cgaiad_keys show local-validator1 -a)
    cgaiad_tx bank send $(cgaiad_keys show local-bank -a) $(cgaiad_keys show local-validator1 -a) 1atom -y

## Relayer

Scripts to init and start IBC relayer instances to connect two chains via channels.

### Requirements

Prepare relayer config file for your chains ([example](./config/relayer_gaiaAB.sh)). Path is bidirectional, so there is no need to create A->B and B->A paths.

### Scripts

* `relayer_init.sh -c path_to_config`
  * Example: `./relayer_init.sh -c config/relayer_gaiaAB.sh`
  * Initializes relayer config, adds chain and path configs.
  * Script sends multiple transactions to create IBC clients and channels (that can take some time).
* `relayer_run.sh -c path_to_config`
  * Example: `./relayer_run.sh -c config/relayer_gaiaAB.sh`
  * Starts multiple relayer instances (one for each path).
  * Each instance is tmux-ed into `{CHAIN1_ID}_{CHAIN2_ID}_relayer_{path_name}` tmux session.
* `stop_relayer.sh -c path_to_config`
  * Example: `./stop_relayer.sh -c config/relayer_gaiaAB.sh`
  * Stops all tmux sessions for relayers.

