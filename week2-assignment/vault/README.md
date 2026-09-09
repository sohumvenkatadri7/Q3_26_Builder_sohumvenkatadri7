# Solana Native SOL Vault Program (Anchor & LiteSVM)

[![Solana](https://img.shields.io/badge/Solana-Anchor%201.1.2-blue?style=flat-square&logo=solana)](https://solana.com)
[![Rust](https://img.shields.io/badge/Rust-1.89.0-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![LiteSVM](https://img.shields.io/badge/Testing-LiteSVM%200.10.0-green?style=flat-square)](https://github.com/LiteSVM/litesvm)
[![License](https://img.shields.io/badge/License-MIT-lightgrey?style=flat-square)](LICENSE)

A robust, production-ready **Native SOL Vault Program** implemented in **Anchor** on Solana. This program provides individual users with a dedicated, program-derived address (PDA) vault to securely deposit, withdraw, and manage native SOL (lamports) with canonical bump caching, PDA cross-program invocation (CPI) signing, rent-exemption validation, and a complete modular LiteSVM integration test suite written in Rust.

---

## 📑 Table of Contents

- [Overview](#-overview)
- [Key Features](#-key-features)
- [Program Architecture & Account Model](#-program-architecture--account-model)
  - [1. State Account (`VaultState`)](#1-state-account-vaultstate)
  - [2. PDA Derivations & Seeds](#2-pda-derivations--seeds)
- [Instruction Lifecycle](#-instruction-lifecycle)
  - [1. `initialize`](#1-initialize)
  - [2. `deposit`](#2-deposit)
  - [3. `withdraw`](#3-withdraw)
  - [4. `close`](#4-close)
- [Error Handling](#-error-handling)
- [Testing Suite (LiteSVM)](#-testing-suite-litesvm)
  - [Running the Tests](#running-the-tests)
  - [Modular Test Breakdown](#modular-test-breakdown)
- [Project Directory Structure](#-project-directory-structure)
- [Security & Best Practices](#-security--best-practices)

---

## 🌟 Overview

The Native Vault program enables any Solana user to establish and manage a sovereign, isolated savings vault controlled purely by programmatic constraints on-chain.

The vault follows a secure lifecycle:
1. **Initialize**: User initializes a `VaultState` account and a `vault` PDA. The user funds the vault with the minimum rent-exemption lamports.
2. **Deposit**: User deposits arbitrary amounts of native SOL into the `vault` PDA via System Program CPI.
3. **Withdraw**: User withdraws a specified amount of native SOL from the `vault` PDA back to their wallet, signed by the program using PDA seeds.
4. **Close**: User drains 100% of the remaining SOL from the `vault` PDA and destroys the `VaultState` data account, refunding all rent lamports back to the user.

```
                    ┌────────────────────────────────────────┐
                    │            INITIALIZE STEP             │
                    │  Creates VaultState & Vault PDA        │
                    │  Funds Vault with Rent-Exempt Lamports │
                    └───────────────────┬────────────────────┘
                                        │
                    ┌───────────────────┴───────────────────┐
                    │                                       │
                    ▼                                       ▼
    ┌──────────────────────────────┐        ┌──────────────────────────────┐
    │         DEPOSIT STEP         │        │        WITHDRAW STEP         │
    │  User transfers SOL → Vault  │        │  Vault PDA signs CPI to User │
    │    Vault balance increases   │        │    Vault balance decreases   │
    └──────────────────────────────┘        └──────────────────────────────┘
                                        │
                                        ▼
                    ┌────────────────────────────────────────┐
                    │               CLOSE STEP               │
                    │  Drains all SOL from Vault → User      │
                    │  Destroys VaultState & refunds rent    │
                    └────────────────────────────────────────┘
```

---

## 🚀 Key Features

- **Native SOL Support**: Interacts directly with the Solana **System Program** for lightweight, zero-overhead native lamport transfers.
- **Canonical Bump Caching**: Stores `vault_bump` and `state_bump` directly in `VaultState` during initialization to eliminate repeated bump derivation compute costs.
- **PDA Signer Seeds Execution**: Implements `CpiContext::new_with_signer` to authorize transfers out of the unkeyed vault PDA.
- **Zero Locked Rent**: The `close` instruction reclaims 100% of the rent lamports from both the `VaultState` account and the `vault` PDA.
- **Modular LiteSVM Test Suite**: Fully covered across separate, modular Rust integration tests using **LiteSVM** for rapid in-memory verification.

---

## 🏛 Program Architecture & Account Model

### 1. State Account (`VaultState`)

The `VaultState` account stores the canonical bump seeds used to validate and sign on behalf of the PDAs:

```rust
// programs/vault/src/state.rs
#[account]
#[derive(InitSpace)]
pub struct VaultState {
    pub vault_bump: u8,
    pub state_bump: u8,
}
```

| Field | Type | Space | Description |
| :--- | :--- | :--- | :--- |
| `vault_bump` | `u8` | 1 byte | Canonical bump seed for the `vault` SystemAccount PDA |
| `state_bump` | `u8` | 1 byte | Canonical bump seed for the `vault_state` data PDA |

### 2. PDA Derivations & Seeds

- **State Account PDA (`vault_state`)**:
  $$\text{Seeds} = \left[\texttt{b"state"},\; \text{user.key}()\right]$$
- **Vault SystemAccount PDA (`vault`)**:
  $$\text{Seeds} = \left[\texttt{b"vault"},\; \text{user.key}()\right]$$

---

## 🔄 Instruction Lifecycle

### 1. `initialize`

Creates the `vault_state` account, records canonical bump seeds, and transfers rent-exemption lamports to the `vault` PDA.

- **Accounts**:
  - `user`: Signer and fee payer (`mut`, `Signer<'info>`).
  - `vault_state`: Initialized data account (`init`, `seeds = [b"state", user]`, `bump`).
  - `vault`: Uninitialized SystemAccount PDA (`mut`, `seeds = [b"vault", user]`, `bump`).
  - `system_program`: Solana System Program.
- **Actions**:
  - Calculates rent-exempt threshold for an empty account (`Rent::get()?.minimum_balance(0)`).
  - Performs CPI transfer from `user` to `vault` to make the vault rent-exempt.
  - Caches `bumps.vault` and `bumps.vault_state` into `VaultState`.

```rust
// programs/vault/src/instructions/initialize.rs
pub fn initialize(&mut self, bumps: &InitializeBumps) -> Result<()> {
    let rent_exempt = Rent::get()?.minimum_balance(self.vault.data_len());
    let cpi_program = self.system_program.key();
    let cpi_accounts = Transfer {
        from: self.user.to_account_info(),
        to: self.vault.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    transfer(cpi_ctx, rent_exempt)?;

    self.vault_state.vault_bump = bumps.vault;
    self.vault_state.state_bump = bumps.vault_state;
    Ok(())
}
```

---

### 2. `deposit`

Allows the user to deposit an arbitrary amount of native SOL into their vault PDA.

- **Parameters**: `amount: u64`
- **Checks**:
  - `amount > 0` (fails with `ErrorCode::InvalidAmount` if zero).
  - Validates `vault_state` and `vault` PDA seeds using the cached bump seeds.
- **Actions**:
  - Transfers `amount` lamports from `user` to `vault` via System Program CPI.

```rust
// programs/vault/src/instructions/deposit.rs
pub fn deposit(&mut self, amount: u64) -> Result<()> {
    require!(amount > 0, ErrorCode::InvalidAmount);

    let cpi_program = self.system_program.key();
    let cpi_accounts = Transfer {
        from: self.user.to_account_info(),
        to: self.vault.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    transfer(cpi_ctx, amount)
}
```

---

### 3. `withdraw`

Allows the user to withdraw a specified amount of native SOL from their vault PDA back to their wallet.

- **Parameters**: `amount: u64`
- **Checks**:
  - `amount > 0` (fails with `ErrorCode::InvalidAmount` if zero).
  - Signer must be the `user`.
- **Actions**:
  - Derives program signer seeds for the `vault` PDA using `[b"vault", user.key(), &[vault_bump]]`.
  - Executes `CpiContext::new_with_signer` to transfer `amount` lamports from `vault` to `user`.

```rust
// programs/vault/src/instructions/withdraw.rs
pub fn withdraw(&mut self, amount: u64) -> Result<()> {
    require!(amount > 0, ErrorCode::InvalidAmount);
    let cpi_program = self.system_program.key();

    let cpi_accounts = Transfer {
        from: self.vault.to_account_info(),
        to: self.user.to_account_info(),
    };

    let user_key = self.user.key();
    let seeds: &[&[u8]] = &[
        VAULT_SEED,
        user_key.as_ref(),
        &[self.vault_state.vault_bump],
    ];
    let signer_seeds: &[&[&[u8]]] = &[&seeds[..]];

    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
    transfer(cpi_ctx, amount)
}
```

---

### 4. `close`

Drains all lamports from the `vault` PDA to the user and closes the `vault_state` account.

- **Accounts**:
  - `user`: Signer receiving the funds and rent refund.
  - `vault_state`: State account marked with `close = user`.
  - `vault`: Vault PDA marked `mut`.
  - `system_program`: Solana System Program.
- **Actions**:
  - Reads total available lamports in the vault (`self.vault.lamports()`).
  - Transfers 100% of vault lamports to `user` via PDA-signed CPI.
  - Anchor automatically zeroes out `vault_state` and transfers its rent refund to `user`.

```rust
// programs/vault/src/instructions/close.rs
pub fn close(&mut self) -> Result<()> {
    let amount = self.vault.lamports();

    let cpi_program = self.system_program.key();
    let cpi_accounts = Transfer {
        from: self.vault.to_account_info(),
        to: self.user.to_account_info(),
    };

    let user_key = self.user.key();
    let seeds: &[&[u8]] = &[
        VAULT_SEED,
        user_key.as_ref(),
        &[self.vault_state.vault_bump],
    ];
    let signer_seeds: &[&[&[u8]]] = &[&seeds[..]];

    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
    transfer(cpi_ctx, amount)
}
```

---

## ⚠️ Error Handling

Custom errors defined in `programs/vault/src/error.rs`:

| Error Code | Error Message | Trigger Condition |
| :--- | :--- | :--- |
| `InvalidAmount` | `"Deposit amount must be greater than zero"` | Attempting to `deposit` or `withdraw` with `amount == 0` |

---

## 🧪 Testing Suite (LiteSVM)

The project includes an in-depth, end-to-end integration test suite written in Rust using [LiteSVM](https://github.com/LiteSVM/litesvm). Each instruction is tested independently in its own test file.

### Running the Tests

```bash
cargo test
```

### Modular Test Breakdown

```
running tests/test_initialize.rs
test test_initialize ... ok

running tests/test_deposit.rs
test test_deposit ... ok
test test_deposit_zero_fails ... ok

running tests/test_withdraw.rs
test test_withdraw ... ok
test test_withdraw_zero_fails ... ok

running tests/test_close.rs
test test_close ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.4s
```

1. **[`test_initialize.rs`](programs/vault/tests/test_initialize.rs)**:
   - Sets up LiteSVM environment, loads compiled `vault.so`, and airdrops 2 SOL to user.
   - Executes `Initialize`.
   - Asserts `vault_state` exists with matching canonical `vault_bump` and `state_bump`.
   - Asserts `vault` balance equals the minimum rent-exemption threshold.

2. **[`test_deposit.rs`](programs/vault/tests/test_deposit.rs)**:
   - Initializes vault and deposits 0.5 SOL (`500_000_000` lamports).
   - Asserts `vault` balance increases by exactly the deposit amount.
   - Asserts attempting to deposit `0` lamports fails with `InvalidAmount`.

3. **[`test_withdraw.rs`](programs/vault/tests/test_withdraw.rs)**:
   - Initializes vault, deposits 0.5 SOL, and withdraws 0.2 SOL (`200_000_000` lamports).
   - Asserts `vault` balance decreases by the withdrawn amount.
   - Asserts `user` wallet balance increases by the withdrawn amount (accounting for tx fee).
   - Asserts attempting to withdraw `0` lamports fails with `InvalidAmount`.

4. **[`test_close.rs`](programs/vault/tests/test_close.rs)**:
   - Initializes vault, deposits 0.5 SOL, and calls `Close`.
   - Asserts `vault_state` account is completely deleted (`svm.get_account(&vault_state).is_none()`).
   - Asserts `vault` PDA balance is drained to 0.
   - Asserts `user` wallet receives both the full vault balance and the refunded state rent.

---

## 📁 Project Directory Structure

```
vault/
├── Anchor.toml                         # Anchor workspace configuration
├── Cargo.toml                          # Cargo workspace definition
├── README.md                           # Comprehensive documentation
├── rust-toolchain.toml                 # Toolchain configuration
└── programs/
    └── vault/
        ├── Cargo.toml                  # Program dependencies & dev-dependencies
        ├── src/
        │   ├── lib.rs                  # Program entrypoints & ID definition
        │   ├── constants.rs            # PDA seed constants (VAULT_SEED, STATE)
        │   ├── state.rs                # VaultState account data structure
        │   ├── error.rs                # Custom program error definitions
        │   ├── instructions.rs         # Instruction submodule re-exports
        │   └── instructions/
        │       ├── initialize.rs       # Initialize handler & accounts
        │       ├── deposit.rs          # Deposit handler & accounts
        │       ├── withdraw.rs         # Withdraw handler & accounts
        │       └── close.rs            # Close handler & accounts
        └── tests/
            ├── test_initialize.rs      # Initialize instruction LiteSVM tests
            ├── test_deposit.rs         # Deposit instruction LiteSVM tests
            ├── test_withdraw.rs        # Withdraw instruction LiteSVM tests
            └── test_close.rs           # Close instruction LiteSVM tests
```

---

## 🔒 Security & Best Practices

- **PDA Authority Isolation**: The `vault` PDA is uniquely derived using `[b"vault", user.key()]`. Only the matching `user` can authorize transfers out of their vault.
- **Canonical Bump Verification**: Bumps are computed once at `initialize` and verified via Anchor constraints (`bump = vault_state.vault_bump`) on subsequent operations, guarding against bump manipulation and saving compute units.
- **Zero Locked Rent**: The `close` instruction drains all lamports and closes the state account, ensuring zero lamports are permanently locked on-chain.
- **Input Validation**: Strict `require!(amount > 0, ErrorCode::InvalidAmount)` checks prevent zero-value transaction spam.
- **Checked CPI Signers**: PDA signer seeds are scoped precisely to the instruction's signer, ensuring no cross-user unauthorized withdrawals.
