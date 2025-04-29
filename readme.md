# 🔄 Delegation Program (Pinocchio Version)

This project reimplements the **Delegation Program** from the [Ephemeral Rollups SDK](https://github.com/magicblock-labs/ephemeral-rollups-sdk) using the **Pinocchio** framework on Solana. Instead of integrating it as part of the SDK, this standalone version is designed for **educational clarity and transparency**.

The goal is to break down and understand the core mechanics of rollup delegation on Solana while showcasing how such systems can be built from scratch using lower-level primitives.

---

## 📚 Background

**Ephemeral Rollups** are designed to scale Solana applications using off-chain execution secured by on-chain proofs and delegation mechanisms.

The original implementation by MagicBlock Labs uses the `solana-program` SDK. This project ports that logic to **Pinocchio** to expose the underlying mechanics without relying on macro-based abstractions.

---

## 🧠 Purpose

- To **learn and teach** how delegation systems work under the hood
- To provide a **Pinocchio-powered alternative** for advanced Solana developers
- To serve as a **reference repo** for building rollup-compatible smart contracts without SDK constraints

---

## ⚙️ Features

- Register and manage delegators and delegates
- Validate authorized delegation instructions
- Reproduce the core logic of MagicBlock's Delegation program
- Built entirely with **Pinocchio** (no Anchor or macros)

---

## 🧪 Usage

### Prerequisites

- Rust (latest stable)
- Solana CLI
- Pinocchio (`cargo add pinocchio`)
- Mollusk (`cargo add mollusk`)

### Build

```bash
cargo build-sbf
```

### Run Tests

```bash
cargo test
```

## 🔍 Inspired By
This project is a Pinocchio-based reimplementation of:

🔗 [Ephemeral Rollups SDK – Delegation Program (MagicBlock Labs)](https://github.com/magicblock-labs/ephemeral-rollups-sdk)

If you're interested in the full rollup architecture and proof system, check out that SDK directly. This repo isolates only the delegation logic.

## 🛠 Tech Stack
Solana – Blockchain runtime

Pinocchio – Low-level framework for Solana smart contracts

TypeScript – For testing with Anchor's Mocha suite

Rust – On-chain logic implementation

## 📜 License
This project is for educational use. MIT License applies.



