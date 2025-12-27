# RustChain 🦀⛓️

([![CI Status](https://github.com/nicoverdin/rust-chain/actions/workflows/rust_ci.yml/badge.svg)](https://github.com/nicoverdin/rust-chain/actions/workflows/rust_ci.yml))

A robust and efficient Layer 1 Blockchain implementation written in Rust from scratch. This project demonstrates advanced concepts in distributed systems, cryptography, and I/O optimization, built purely for educational and engineering purposes.

## 🚀 Key Features

- **Consensus Mechanism:** Proof of Work (PoW) algorithm with dynamic difficulty embedded within block metadata.
- **Efficient Persistence:** Custom storage engine based on **Append-Only Logs** using NDJSON, ensuring **O(1)** write complexity and crash consistency.
- **Transaction Management:** Implemented a **Mempool** (Memory Pool) to decouple transaction ingestion from block mining, simulating high-throughput environments.
- **Data Integrity:** Full cryptographic validation (Hash linkage) to guarantee the immutability of the ledger history.
- **Type Safety:** Leveraging Rust's strict type system to prevent invalid states at compile time.
- **Code Quality (CI/CD):** Automated GitHub Actions workflow that executes `cargo test` and `clippy` on every commit to enforce code stability and best practices.

## 🛠️ Tech Stack

- **Language:** Rust (2021 Edition)
- **Serialization:** Serde / Serde JSON
- **Cryptography:** SHA-256 (`sha2` crate)
- **Persistence:** File System (Buffered I/O with Append-Only logic)

## 🏗️ Architecture & Engineering Decisions

### 1. Persistence Strategy: Append-Only vs. Rewrite
**Problem:** Initially, the chain persistence was handled by serializing the entire `Blockchain` struct to a JSON file. As the chain grew, the write complexity became **O(N)**, causing significant latency.
**Solution:** Refactored the storage layer to use an **Append-Only Log** model. New blocks are appended to the end of a file (`history.db`) using newline-delimited JSON. This reduced write latency to **O(1)** and improved data safety against crashes (atomic-like writes).

### 2. Stateless Proof of Work (PoW)
The consensus mechanism utilizes a Hashcash-style PoW.
- Blocks are self-contained units; they include their own `difficulty` and `nonce`.
- This design allows for **stateless validation**: a node can verify the validity of a specific block without needing to query the global configuration state at that specific point in time.

### 3. Mempool & Mining Separation
To mimic real-world distributed ledgers, the architecture separates the "Submit Transaction" action from the "Mine Block" action.
- Transactions enter an in-memory buffer (Mempool).
- The miner aggregates a batch of pending transactions to construct a block, maximizing network throughput.

## ⚡ How to Run

Prerequisites: Ensure `cargo` and `rustc` are installed.

```bash
# 1. Clone the repository
git clone [https://github.com/nicoverdin/rust-chain](https://github.com/nicoverdin/rust-chain)
cd rust-chain

# 2. Run the node 
# (Note: Delete 'history.db' if you are migrating from a previous version)
cargo run
```

## 🗺️ Roadmap

- [x] Block Structure & Hashing Logic
- [x] Proof of Work Consensus
- [x] Optimized Disk Persistence (Append-Only)
- [x] Transactions & Mempool
- [ ] Digital Signatures (Elliptic Curve Cryptography)
- [ ] P2P Network Implementation (libp2p)
- [ ] CLI Wallet Interface