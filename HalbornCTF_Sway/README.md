## Sway CTF

# SimpleLiquidityPool CTF Challenge

Welcome to the **SimpleLiquidityPool** CTF challenge! Your task is to analyze and exploit a smart contract to drain its funds using the provided test environment.

## Challenge Overview

You are provided with a Sway smart contract that implements a simple liquidity pool. The goal of this challenge is to find and exploit a vulnerability in the contract that allows you to withdraw all the **BASE_TOKEN** from the contract into your own wallet.

## Setup Instructions

### Prerequisites

- **Rust Toolchain**: Ensure you have the Rust toolchain installed. You can install it from [rustup.rs](https://rustup.rs/).
- **Forc (Fuel Orchestrator)**: Install Forc, the Fuel toolchain, by following the instructions at [Fuel Labs Documentation](https://fuellabs.github.io/forc/latest/).
- **Cargo**: Cargo is the Rust package manager and is included with the Rust toolchain.

# Building the Project
Build the smart contract using Forc:

```bash
forc build
```

# Running the Tests
Run the tests using Cargo to start the challenge:
```bash
cargo test -- --nocapture
```

# Understanding the Test Environment
The test environment sets up a scenario where:
  - Admin Wallet (`wallet_admin`):
    - deploys and funds the smart contract with 10,000,000 units of BASE_TOKEN.
    - You do not have access to this wallet and cannot use it.

  - Attacker Wallet (`wallet_attacker`):
    - Your wallet for this challenge.
    - Funded with enough gas tokens to perform transactions and mint LP_TOKENS.
    - Initially has 0 units of BASE_TOKEN.
    - Has access to the contract address and its ABI.

  - Contract Instance:
    - Deployed smart contract that holds 10,000,000 units of BASE_TOKEN.
    - Contains a vulnerability that you need to exploit.
  
# Your Objective
  - Goal: Exploit the vulnerability in the SimpleLiquidityPool contract to withdraw all 10,000,000 units of BASE_TOKEN into your attacker wallet (wallet_attacker).
  - Constraints:
    - You can only modify the test_liquidity_pool function in the tests/harness.rs file.
    - Your exploit should be written within the test_liquidity_pool function.
    - Do not modify the get_contract_instance function or any other part of the code.
    - Use only the provided wallet_attacker and the contract's public methods.

# Getting Started

1. Analyze the Contract:
   1. Review the contract's ABI and understand its public methods.
   2. Identify potential vulnerabilities in how the contract handles deposits and withdrawals.
2. Craft Your Exploit:
   1. Develop an exploit that takes advantage of the identified vulnerability.
   2. Your exploit should be written within the test_liquidity_pool function.
   3. The test should pass if your exploit successfully withdraws all the BASE_TOKEN from the contract into your wallet.
3. Test Your Exploit:
   1. Run the tests using `cargo test -- --nocapture` to see the results.
   2. The test should pass if your exploit successfully withdraws all the BASE_TOKEN from the contract into your wallet.

# Helpful Information
   - Contract Address: Accessible via the contract_id variable within the test function.
   - Attacker Wallet: Represented by wallet_attacker, which is pre-funded with gas tokens.
   - BASE_TOKEN Asset ID: Provided as base_asset_id, token to be deposited on the contract to mint LP_TOKENS
   - LP_TOKEN_SUB_ID: Used for calculations related to liquidity pool tokens.
  

Good luck, and happy hacking!


# Files and Structure
The project has the following important files:
   - src/main.sw: The Sway smart contract code. (do not need any modifs)
   - tests/harness.rs: The Rust test harness where you will write your exploit in the test_liquidity_pool function. (obviously need modifs)
   - forc.toml: The Forc configuration file. (do not need modifs)
   - Cargo.toml: The Cargo configuration file for Rust dependencies. (do not need any modifs)



