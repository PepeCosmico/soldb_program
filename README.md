# 🧱 SolDB Program

**SolDB** is an on-chain program written in **Rust** for the **Solana** blockchain.  
Its goal is to provide a simple, table-based key-value database fully managed within PDA accounts controlled by the program itself.

---

## 📘 Overview

The program implements a hierarchical storage model where:

- Each **table** is a PDA account derived from the table name and the creator’s public key.
- Each **value** is a PDA account derived from the table and the record key.
- Data is serialized using **Borsh** for compact and deterministic encoding.

Main instructions implemented:
- **InitTable** → Creates a new table.
- **Put** → Inserts or updates a key-value pair under an existing table.
- **Delete** → Deletes a specific record from a table.
- (Future extensions may include read or cleanup operations).

---

## ⚙️ Project structure

```
soldb_program/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── entrypoint.rs
│   ├── processor.rs
│   ├── instructions.rs
│   ├── accounts.rs
│   ├── error.rs
│   └── utils.rs
└── tests/
    ├── init_table_tests.rs
    ├── put_tests.rs
    ├── delete_tests.rs
    └── utils.rs
```

- **instructions.rs** → Defines the available instructions (`InitTable`, `Put`, `Delete`).
- **processor.rs** → Contains the core logic for each instruction.
- **accounts.rs** → Defines the data structures `SolTable` and `SolValue`.
- **error.rs** → Custom error definitions (`SolDbError`).
- **tests/** → Integration tests using `solana-program-test`.

---

## 🧰 Technologies

- **Rust** (nightly, Solana-compatible)
- **Solana SDK** `v2.x`
- **Borsh** for binary serialization
- **Program Test Framework** for local testing
- **Cargo Make** / **Makefile** for build automation

---

## 🚀 Build & Test

1. **Install Solana CLI**  
   ```bash
   sh -c "$(curl -sSfL https://release.solana.com/v1.18.0/install)"
   ```

2. **Build the on-chain program (SBF)**  
   ```bash
   cargo build-sbf --manifest-path=./programs/soldb_program/Cargo.toml
   ```

3. **Run integration tests**  
   ```bash
   cargo test-sbf
   ```

4. **Optional: native unit tests**
   ```bash
   cargo test
   ```

---

## 🧪 Example instruction

```rust
let instr = SolDbIntructions::InitTable(InitTable {
    name: "Users".to_string(),
    bump,
});
```

Each instruction is serialized with `borsh` and sent as part of a Solana `Transaction`.  
PDAs are derived using seeds like `["table_name", owner_pubkey]`.

---

## 🧩 Storage layout

```text
┌────────────────────────┐
│ Owner Account (signer) │
└────────────┬───────────┘
             │
             ▼
      ┌─────────────┐
      │  Table PDA  │  ← derived from (table_name, owner)
      └─────────────┘
             │
             ▼
      ┌─────────────┐
      │  Value PDA  │  ← derived from (key, table_pda, owner)
      └─────────────┘
```

Each PDA stores Borsh-serialized data:  
- `SolTable { name: String }`  
- `SolValue { val: Vec<u8> }`

---

## 📄 License

This project is licensed under the **MIT License**.  
See the [`LICENSE`](./LICENSE) file for details.

---

## ✍️ Author

**Pedro Llinás Ferrer**  
Developer of SolDB – Universidad Politécnica de Madrid (ETSISI)

---

## 🌐 Future improvements

- Add **Get/Scan** operations for direct reads  
- Dynamic PDA resizing support  
- Integration with off-chain clients in Rust or TypeScript
