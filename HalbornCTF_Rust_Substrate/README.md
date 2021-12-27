
# Malborn Chain

A Blockchain node for the Malborn Chain to connect and secure the next trillion things. It is still work in progress. Only the initial pallets are done. 

> Built on [Substrate](https://substrate.dev).
# Scope

Everything in the pallets folder.

# Development

## Building
```
cargo build
```

## Testing
```
cargo test --all
```

# Usage
```
./target/debug/malborn-chain purge-chain --dev # Purge old chain data
./target/debug/malborn-chain --dev             # Run a single node testnet
```

To save chain data in the temporary storage:

```
./target/debug/malborn-chain --dev --tmp
```

Navigate to: 

```
https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/explorer
```

You can execute extrinsics at:

```
https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/extrinsics
```
If types are not resolved, please paste types from the **types.json** to the: 

```
https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/settings/developer
```