# RustChain 🦀⛓️

[![CI Status](https://github.com/nicoverdin/rust-chain/actions/workflows/rust_ci.yml/badge.svg)](https://github.com/nicoverdin/rust-chain/actions/workflows/rust_ci.yml)

A robust, efficient, and fully decentralized Layer 1 Blockchain implementation written in Rust. This project demonstrates advanced concepts in distributed systems, asynchronous networking, cryptography, and I/O optimization.

## 🚀 Key Features

- **Decentralized P2P Network:** Fully functional Peer-to-Peer layer using **libp2p**. Nodes automatically discover each other (mDNS) and propagate transactions using a Gossip protocol.
- **Async & Concurrency:** Built on the **Tokio** runtime, allowing the node to handle user input, network events, and mining operations concurrently without blocking.
- **Consensus Mechanism:** Proof of Work (PoW) algorithm with dynamic difficulty embedded within block metadata.
- **Cryptographic Identity:** Wallet implementation using **Elliptic Curve Cryptography (Ed25519)** to sign transactions, ensuring non-repudiation and preventing identity theft.
- **Efficient Persistence:** Custom storage engine based on **Append-Only Logs** using NDJSON, ensuring **O(1)** write complexity and crash consistency.
- **Transaction Management:** Implemented a **Mempool** to decouple transaction ingestion from block mining.
- **Data Integrity:** Full cryptographic validation (Hash linkage) to guarantee the immutability of the ledger history.

## 🛠️ Tech Stack

- **Language:** Rust (2021 Edition)
- **Async Runtime:** Tokio
- **Networking:** libp2p (TCP, mDNS, Gossipsub, Noise, Yamux)
- **Serialization:** Serde / Serde JSON
- **Cryptography:** SHA-256 (`sha2`), Ed25519 (`ed25519-dalek`)
- **Persistence:** File System (Buffered I/O with Append-Only logic)

## 🏗️ Architecture & Engineering Decisions

### 1. Event-Driven P2P Architecture (Tokio + libp2p)
**Problem:** A blocking, synchronous architecture cannot handle network traffic and user input simultaneously.
**Solution:** Migrated the core engine to an asynchronous model using **Tokio**.
- **Swarm:** The node manages a `Swarm` of peers using `libp2p`.
- **Discovery:** Uses **mDNS** for automatic local peer discovery (Zero-Conf).
- **Propagation:** Uses **Gossipsub** (a pub/sub protocol) to efficiently broadcast transactions and blocks to the entire network mesh.
- **Concurrency:** The Blockchain state is wrapped in an `Arc<Mutex<Blockchain>>`, allowing thread-safe access from both the UI thread and the Network thread.

### 2. Persistence Strategy: Append-Only vs. Rewrite
**Problem:** Serializing the entire chain becomes **O(N)**, causing latency.
**Solution:** Refactored to an **Append-Only Log** model. New blocks are appended to `history.db` using newline-delimited JSON. This ensures **O(1)** write complexity and data safety.

### 3. Zero-Trust Security Model
To prevent transaction spoofing, the system enforces a strict signature verification process.
- **Algorithm:** Uses **Ed25519** for high-performance digital signatures.
- **Flow:** Users sign transaction hashes with their Private Key. Nodes validate the signature against the public key before adding it to the Mempool.

## ⚡ How to Run (P2P Demo)

To see the decentralized network in action, you need to run at least two nodes.

### Terminal 1 (Node A)
```bash
cargo run
# Copy the Public Address displayed (e.g., AAAA...)
```

### Terminal 2 (Node B)
```bash
cargo run
# Wait for the message: "👋 Nuevo vecino encontrado..."
```

### Test Connectivity
1. In **Terminal 1**, select Option `1` (Send Money).
2. Paste the address of **Node B**.
3. Enter an amount (e.g., 50).
4. Watch **Terminal 2**: You will see the transaction arrive via the network automatically (`🔀 Recibida Tx...`).

### Test Block Synchronization (Mining)
1. In **Terminal 1**, select Option `2` (Mine Block).
2. Wait for the mining process to finish.
3. Watch **Terminal 2**: It will receive the new block, validate the Proof of Work, and append it to its local chain automatically.

## 🗺️ Roadmap

- [x] Block Structure & Hashing Logic
- [x] Proof of Work Consensus
- [x] Optimized Disk Persistence (Append-Only)
- [x] Transactions & Mempool
- [x] Digital Signatures (Elliptic Curve Cryptography)
- [x] P2P Network (Discovery & Gossipsub)
- [x] Block Propagation & Chain Synchronization
- [ ] Wallet CLI Interface (Key Management)