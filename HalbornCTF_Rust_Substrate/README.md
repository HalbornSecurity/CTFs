# Capture The Flag (CTF) - Substrate Pallet Code Review

## Introduction

Welcome to the Substrate Pallets CTF! This CTF focuses on two specific pallets: Pause Pallet and Allocation Pallet. The goal is to uncover vulnerabilities and security issues within the codebase through careful examination and testing.

## Pallets in Scope

1. **Pause Pallet:**
    - Description: The Pause Pallet allows pausing and unpausing certain functionality within the blockchain.
    - Source Code: [pause-pallet](./pallets/pause)

2. **Allocation Pallet:**
    - Description: The Allocation Pallet manages resource allocation within the blockchain, crucial for maintaining balance and fairness.
    - Source Code: [allocations-pallet](./pallets/allocations)

## Getting Started

To participate in the CTF, follow these steps:

1. Clone the repository:
```shell
git clone https://github.com/HalbornSecurity/CTFs.git
cd CTFs/Substrate/Pallets
```

2. Identify and exploit vulnerabilities in the Pause and Allocation pallets.

3. Write a detailed report for all the issues.

## Bug Severity Levels

- **Low:** Minor issues that have minimal impact.
- **Medium:** Issues that may have a moderate impact on security. (POC Required)
- **High:** Significant issues that pose a notable security risk. (POC Required)
- **Critical:** Critical vulnerabilities that pose a severe security risk. (POC Required)

## Reporting Bugs

1. Provide a clear title and description of the issue.

2. Include the severity level and steps to reproduce.

3. If applicable, provide a Proof of Concept (PoC) code in the unit tests.
