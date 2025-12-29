# RustChain 🦀⛓️

[![CI Status](https://github.com/nicoverdin/rust-chain/actions/workflows/rust_ci.yml/badge.svg)](https://github.com/nicoverdin/rust-chain/actions/workflows/rust_ci.yml)

A robust, efficient, and fully decentralized Layer 1 Blockchain implementation written in Rust. This project demonstrates advanced concepts in distributed systems, asynchronous networking, cryptography, and I/O optimization.

## 🚀 Key Features

- **Decentralized P2P Network:** Fully functional Peer-to-Peer layer using **libp2p**. Nodes automatically discover each other (mDNS) and propagate both Transactions and Blocks.
- **Consensus Engine:** Implements the **Longest Chain Rule**. Nodes automatically resolve forks by adopting the chain with the most accumulated work (Height).
- **Persistent Identity:** Your wallet (Private Key) is encrypted and saved locally to `wallet.key`. You maintain your balance and reputation across restarts.
- **Async & Concurrency:** Built on the **Tokio** runtime, allowing the node to handle user input, network events, and mining operations concurrently without blocking.
- **Efficient Persistence:** Custom storage engine based on **Append-Only Logs** using NDJSON, ensuring **O(1)** write complexity and crash consistency.
- **Data Integrity:** Full cryptographic validation (Ed25519 Signatures & SHA-256 Hashing) to guarantee the immutability of the ledger history.
- **Adaptive Difficulty:** The network self-regulates to maintain a constant block generation time (Target: 4 seconds). It increases difficulty if blocks are mined too fast and decreases it if the network is sluggish.
- **High-Performance State System:** Uses an in-memory `HashMap` cache to track user balances. Balance lookups are **O(1)** (instant), regardless of blockchain size.
- **Economic Security:** strictly enforces solvency rules. Transactions attempting to spend more than the available balance are rejected at the Mempool level.

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
- **Propagation:** Uses **Gossipsub** (a pub/sub protocol) to efficiently broadcast transactions, blocks, and full chain states.

### 2. Consensus: Longest Chain Rule
**Problem:** In a distributed system, two miners can solve a block simultaneously (Fork).
**Solution:** The system follows the Nakamoto Consensus rule. When a node receives a valid chain that is longer than its current one, it performs a **Chain Reorganization**, discarding its local history in favor of the network's majority truth.

### 3. Persistent Wallet Management
**Problem:** Generating a new identity on every run makes holding balances impossible.
**Solution:** Implemented a secure `WalletManager` that serializes the Ed25519 keypair to disk (`wallet.key`). The system automatically loads existing identities or generates new ones if missing.

### 4. Dynamic Difficulty Adjustment (Homeostasis)
**Problem:** If network hashrate increases (better hardware or more miners), blocks would be mined instantly, flooding the network with data.
**Solution:** The system monitors the time taken to mine the last **5 blocks**.
- **Target:** 1 block every 4 seconds (20 seconds per epoch).
- **Adjustment:**
  - If time < 10s (Too Fast) → Difficulty increases (+1).
  - If time > 40s (Too Slow) → Difficulty decreases (-1).

### 5. Account Model & State Caching
**Problem:** Calculating a user's balance by iterating through the entire history of blocks is **O(N)** and inefficient as the chain grows.
**Solution:** The system maintains a localized `State` (HashMap) of all accounts.
- **Update:** When a block is mined or received, the state updates incrementally.
- **Query:** `get_balance` is **O(1)**.
- **Validation:** Transactions are validated against this state before entering the Mempool, preventing Double Spending and Insufficient Funds errors.

## ⚡ How to Run (P2P Demo)

To see the decentralized network in action, you need to run at least two nodes.

### Terminal 1 (Node A - The Leader)
```bash
cargo run
# This node will hold the Longest Chain
```

### Terminal 2 (Node B - The Follower)
```bash
cargo run
# This node will start with a shorter chain
```

### Test Consensus (Chain Reorg)
1. **In Terminal 1:** Mine 3 blocks (Option `2` three times). Node A now has Height 4.
2. **In Terminal 2:** Mine 1 block. Node B has Height 2. (They are now forked).
3. **In Terminal 1:** Select Option `7` (BROADCAST FULL CHAIN).
4. **Watch Terminal 2:** It will analyze the incoming chain, verify it is longer and valid, and replace its local database automatically.
   > `🔄 CHAIN SYNC COMPLETE: Local chain replaced.`

## 🗺️ Roadmap

- [x] Block Structure & Hashing Logic
- [x] Proof of Work Consensus
- [x] Optimized Disk Persistence (Append-Only)
- [x] Transactions & Mempool
- [x] Digital Signatures (Elliptic Curve Cryptography)
- [x] P2P Network (Discovery & Gossipsub)
- [x] Persistent Wallet (Key Storage)
- [x] Consensus Algorithm (Longest Chain Rule & Sync)
- [x] Dynamic Difficulty Adjustment (Target: 4s/block)
- [x] Account Model Optimization (O(1) Balance)
- [x] Double Spend Protection
- [ ] Wallet CLI Interface (Advanced Key Management)