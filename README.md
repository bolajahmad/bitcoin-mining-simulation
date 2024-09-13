# Blockchain Protocol Development: Mining a block

## Overview

This project involves the simulation of mining process of a block, which includes validating and including transactions from a given set of transactions.
The repository contains a folder `mempool` which contains JSON files.
These files represent individual transactions. The goal is to successfully mine a block by including some of these transactions, following the specific requirements outlined below.

> [!NOTE] 
> It only required to do basic validation checks. Signature validation may be included.


## Objective

The primary objective is to write a script that processes a series of transactions, validates them, and then mines them into a block. The output of the script is a file named `out.txt` that follows a specific format.

The solution is written in the following language:
- [javascript](./test/sanity-checks.spec.ts): For unit tests
- [rust](./rust/src/main.rs): The implementation

## Requirements

### Input

- The folder named `mempool` contains several JSON files. Each file represents a transaction that includes all necessary information regarding the transaction.

### Output

The script generates an output file named `out.txt` with the following structure:
- First line: The block header.
- Second line: The serialized coinbase transaction.
- Following lines: The transaction IDs (txids) of the transactions mined in the block, in order. The first txid should be that of the coinbase transaction

### Difficulty Target
The difficulty target is `0000ffff00000000000000000000000000000000000000000000000000000000`. This is the value that the block hash must be less than for the block to be successfully mined.

## Execution
To test the solution locally:
- Uncomment the lines in [run.sh](./run.sh).
- Execute [`local.sh`](./local.sh).

If the code works, you will see the test completed successfully.

### Plagiarism Policy

The solution has been devised by reviewing the documentation of Bitcoin as specified by popular sources like [Learn bitcoin](https://learnmeabitcoin.com/) and under the guide of the [Rust-for-bitcoiner](https://bitcoin-dev-project.gitbook.io/rust-for-bitcoiners) camp organized by [Chaincode labs](https://chaincode.com/). The solution was developed by learning these resources. Attempts were made to avoid plagiarizing online contents except for learning and improvement purposes.

### AI Usage Disclaimer
I used AI tools like ChatGPT/Copilot to gather information and explore alternative approaches, but didn't rely on AI for any complete solutions. Verified andd validate any insights obtained and maintained a balance between AI assistance and independent problem-solving.
