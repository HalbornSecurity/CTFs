## Solidity EVM CTF

There are 3 contracts in the `src`folder:
- HalbornLoans.sol
- HalbornNFT.sol
- HalbornToken.sol

There are several vulnerabilities that were found in live projects. 

**We do not want a report that is full of low/informational issues such as missing zero address checks or floating pragmas.
We are looking for engineers who can fully understand the purpose of these contracts and can find all critical/high issues in them.**

Most Halborn engineers use Foundry for manual testing. We would really value any critical/high finding that also have a Foundry test attached as a Proof of Concept to reproduce the issue. The Foundry project was already created.

https://github.com/foundry-rs/foundry

**Hint**: The CTF contracts (among all) contain at least 5 different critical issues.
