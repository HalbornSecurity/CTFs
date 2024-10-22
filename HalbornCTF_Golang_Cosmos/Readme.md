# Golang Cosmos CTF
## Overview

Welcome to the Halborn Golang Cosmos CTF! Your task is to analyze and exploit at least 7 different issues in the protocol.

The primary flow of the system is as follows:

1. **Minting HAL Tokens**: Users can mint HAL tokens by providing specified collateral.

2. **Ticket Creation**: Users can use HAL tokens to create tickets on the Halborn blockchain, paying for each letter of the ticket description using HAL tokens.

3. **Redeeming Collateral**: When users no longer want to participate, they can redeem HAL tokens back to the original collateral. After a staking period, the tokens are distributed automatically by the chain.

## Get started
**Versions used**
```
Ignite CLI version:             v28.5.3
Cosmos SDK version:             v0.50.9
Your go version:                go version go1.23.2
```

```
ignite chain serve
```