# Solana Token-2022 Remittance Stablecoin

An institutional-grade, regulatory-compliant remittance stablecoin program built on Solana using **Anchor** and **Token Extensions (Token-2022 / `spl-token-2022`)**.

This program demonstrates how to combine advanced Token Extensions—such as Transfer Fees, Default Account State (KYC gating), Metadata Pointer, Mint Close Authority, Permanent Delegate (Regulatory Seizure), and Confidential Transfers—into a unified smart contract architecture.

---

## Architecture Overview

```
                      +------------------------------------------+
                      |         Token-2022 Mint Account          |
                      +------------------------------------------+
                      | Base: Decimals, Mint Auth, Freeze Auth   |
                      +------------------------------------------+
                      | TLV Extensions:                          |
                      |  - TransferFeeConfig (Dynamic Fees)      |
                      |  - MetadataPointer (Self-referencing)    |
                      |  - DefaultAccountState (Frozen for KYC)  |
                      |  - MintCloseAuthority (Decommissioning)  |
                      |  - PermanentDelegate (Compliance/Seize)  |
                      |  - ConfidentialTransferMint (ZKP Privacy)|
                      |  - ConfidentialTransferFeeConfig        |
                      +------------------------------------------+
                                           |
                   +-----------------------+-----------------------+
                   |                                               |
                   v                                               v
     +---------------------------+                   +---------------------------+
     |   Alice Token Account     |   Transfer Fee    |    Bob Token Account      |
     | (Frozen -> KYC Thawed)    | ----------------> |  (Withheld Fee Collected) |
     +---------------------------+                   +---------------------------+
```

---

## Key Features & Extensions

| Extension | Purpose | Implementation Detail |
| :--- | :--- | :--- |
| **`TransferFeeConfig`** | Enforces protocol transfer fees on transactions | Dynamic basis points and max fee cap per transfer; fee collected to mint or withdraw authority. |
| **`DefaultAccountState`** | KYC / Compliance gating | New token accounts are initialized in the `Frozen` state by default until KYC approval. |
| **`MetadataPointer`** | Token metadata binding | Points to the mint account itself for integrated on-chain metadata resolution. |
| **`MintCloseAuthority`** | Lifecycle management & sunsetting | Enables closing empty mint accounts to reclaim rent lamports. |
| **`PermanentDelegate`** | Legal & regulatory enforcement | Allows designated compliance authority to transfer or burn tokens for AML/CFT enforcement. |
| **`ConfidentialTransferMint`** | Zero-Knowledge confidential balances | Supports ElGamal encrypted balances with manual approval policies. |
| **`ConfidentialTransferFeeConfig`** | Confidential fee compatibility | Required companion extension when stacking confidential transfers with transfer fees. |

---

## Instructions

### 1. `initialize_stablecoin`
Creates and initializes a new stablecoin mint with the initial extension stack:
- Computes exact TLV allocation size using `ExtensionType::try_calculate_account_len::<Mint>()`.
- Allocates space and transfers rent via System Program CPI.
- Initializes `TransferFeeConfig`, `MetadataPointer`, `DefaultAccountState::Frozen`, and `MintCloseAuthority`.
- Finalizes the mint via `initialize_mint2`.

### 2. `transfer_with_fee`
Executes token transfers with dynamic protocol fee calculation:
- Unpacks mint data safely using `StateWithExtensions::<Mint>::unpack`.
- Reads `TransferFeeConfig` and calculates the dynamic epoch fee for `Clock::get()?.epoch`.
- Invokes `transfer_checked_with_fee` CPI.

### 3. `thaw_kyc_account`
Allows the `freeze_authority` to thaw an individual token account once the holder passes off-chain KYC/AML verification.

### 4. `reissue_confidential_mint`
Re-issues or deploys a confidential mint with full 7-extension compliance stack:
- Stacks `TransferFeeConfig`, `MetadataPointer`, `DefaultAccountState`, `MintCloseAuthority`, `PermanentDelegate`, `ConfidentialTransferMint`, and `ConfidentialTransferFeeConfig`.
- Configures manual approval policy (`auto_approve_new_accounts = false`).

### 5. `approve_confidential_account`
Enables the `confidential_transfer_authority` to approve individual token accounts for confidential transfers after audit/verification.

### 6. `close_mint`
Closes a decommissioned mint using the `close_authority`, refunding all rent lamports to the destination account.

---

## Project Structure

```
stablecoin/
├── Anchor.toml
├── Cargo.toml
└── programs/
    └── stablecoin/
        ├── Cargo.toml
        ├── src/
        │   ├── lib.rs          # Core Anchor program and instructions
        │   └── error.rs        # Custom program error codes
        └── tests/
            └── test_stablecoin.rs # LiteSVM integration test suite
```

---

## Getting Started

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (1.89+ or compatible Solana toolchain)
- [Solana CLI](https://docs.solanalabs.com/cli/install)
- [Anchor CLI](https://www.anchor-lang.com/docs/installation)

### Build the Program

Compile the Solana SBF binary:
```bash
cargo build-sbf
```

### Run Tests

The test suite runs against **LiteSVM** for fast, deterministic in-process simulation:
```bash
cargo test --test test_stablecoin
```

#### Test Coverage:
- `test_task_1_initialize_stablecoin_mint`: Validates 4-extension allocation, TLV ordering, and fee configuration.
- `test_task_2_3_transfer_with_fee`: Verifies safe state reading via `StateWithExtensions`, dynamic epoch fee calculation, and fee withholding.
- `test_task_4_thaw_kyc_account`: Verifies default frozen state and compliance thawing.
- `test_task_5_reissue_confidential_mint`: Verifies 7-extension confidential mint setup with permanent delegate.
- `test_task_6_approve_confidential_account`: Verifies manual approval workflow for confidential accounts.
- `test_close_mint`: Verifies mint decommissioning and rent reclamation.

---

## Security & Best Practices

1. **Extension Initialization Order**: All extension initialization instructions are executed **before** `initialize_mint2`. In Token-2022, extension ordering is immutable once the base mint is initialized.
2. **TLV Account Sizing**: Space calculation is computed strictly via `ExtensionType::try_calculate_account_len` to avoid under-allocation runtime errors.
3. **Safe State Reading**: Mint state is unpacked strictly using `StateWithExtensions::<Mint>::unpack(&mint_data)` rather than naive slice deserialization, guaranteeing correct TLV header offsets.
4. **CPI Helpers**: Account creations and extension initializations use Anchor's type-safe `CpiContext` and CPI wrappers.
