# Solana SPL Token & MPL Core NFT Suite

A comprehensive TypeScript repository for managing the complete lifecycle of **SPL Tokens** and **Metaplex Core (MPL-Core) NFTs** on Solana Devnet using `@solana/kit`, `@solana-program/token`, and `@metaplex-foundation/umi`.

---

## 📋 Assignment Tasks Overview

| # | Task | Implementation Script | Status |
|---|---|---|:---:|
| **1** | **Mint and transfer your own SPL token** | `src/spl/spl_init.ts`, `src/spl/spl_metadata.ts`, `src/spl/spl_mint.ts`, `src/spl/spl_transfer.ts` | ✅ Completed |
| **2** | **Mint an NFT using MPL Core** | `src/nft/nft_image.ts`, `src/nft/nft_metadata.ts`, `src/nft/nft_mint.ts` | ✅ Completed |
| **3** | **Update the NFT's name and metadata as Update Authority** | `src/nft/nft_update.ts` | ✅ Completed |

---

## 📁 Repository Structure

```tree
spl-nft-q326/
├── devnet-wallet.json       # Devnet keypair array for fee payer & authority
├── generug.png              # NFT asset image uploaded to decentralized storage
├── package.json             # NPM dependencies and script shortcuts
├── tsconfig.json            # TypeScript configuration
├── src/
│   ├── spl/                 # Task 1: SPL Token scripts
│   │   ├── spl_init.ts      # 1. Initialize SPL token mint account
│   │   ├── spl_metadata.ts  # 2. Attach on-chain metadata via Metaplex Token Metadata V3
│   │   ├── spl_mint.ts      # 3. Create ATA and mint tokens
│   │   └── spl_transfer.ts  # 4. Create recipient ATA and transfer tokens (checked)
│   └── nft/                 # Tasks 2 & 3: Metaplex Core NFT scripts
│       ├── nft_image.ts     # 1. Upload raw image to Irys decentralized storage
│       ├── nft_metadata.ts  # 2. Upload metadata JSON to Irys decentralized storage
│       ├── nft_mint.ts      # 3. Mint on-chain Metaplex Core Asset
│       └── nft_update.ts    # 4. Fetch asset, upload updated metadata, & update on-chain
└── README.md
```

---

## 🛠️ Setup & Installation

### 1. Prerequisites
- **Node.js**: `v18+` or `v20+`
- **npm** or **yarn** / **pnpm**
- A funded **Solana Devnet Wallet** (`devnet-wallet.json`)

### 2. Install Dependencies
```bash
npm install
```

### 3. Wallet Configuration
Ensure your devnet keypair JSON is placed in the project root:
```
./devnet-wallet.json
```
*(Format: JSON array of 64 bytes, e.g. `[174, 23, ...]`).*

---

## 🪙 Task 1: SPL Token Lifecycle

The SPL token lifecycle covers mint creation, on-chain metadata attachment, token minting into an Associated Token Account (ATA), and transferring tokens to another wallet.

### 1.1 Initialize Mint Account
Creates a new SPL Token mint account with **6 decimals** using `@solana/kit` and `@solana-program/system` / `@solana-program/token`.

- **Command:**
  ```bash
  npm run spl:init
  ```
- **Terminal Output:**
  ```text
  Sending transaction and waiting for confirmation...
  mint address: 4xaTz8bEzFaYE1eXpaVS5R1GVAafZxiCXt6Phnu7Ka3r. Transaction Signature: 2PknoC661jNHZR9zQtCuVUY8R5iSxLCVbM1aroWpL9ZJCGUFCdJsbzxesD9VTXe6yXcBaKWAmmt1cZD1gwJkweY8
  ```
- **Solana Explorer:** [View Mint Initialization Transaction](https://explorer.solana.com/tx/2PknoC661jNHZR9zQtCuVUY8R5iSxLCVbM1aroWpL9ZJCGUFCdJsbzxesD9VTXe6yXcBaKWAmmt1cZD1gwJkweY8?cluster=devnet)

---

### 1.2 Create Token Metadata
Attaches token metadata (`Turbin3 Sohum`, symbol `TURB3`) to the newly created mint using Metaplex Token Metadata program via UMI.

- **Command:**
  ```bash
  npm run spl:metadata
  ```
- **Terminal Output:**
  ```text
  signature:  3pukoPCpzMqGXcFmMSsH5tZPu9BF1ocaDKMyE7Q2Pqn5phQSbh7syAfD52pTZLyJGzq2r5Zz4XNvB96J2cVKvCdg
  ```
- **Solana Explorer:** [View Metadata Creation Transaction](https://explorer.solana.com/tx/3pukoPCpzMqGXcFmMSsH5tZPu9BF1ocaDKMyE7Q2Pqn5phQSbh7syAfD52pTZLyJGzq2r5Zz4XNvB96J2cVKvCdg?cluster=devnet)

---

### 1.3 Mint Tokens to Associated Token Account (ATA)
Finds or derives the sender's ATA, initializes it, and mints `1,000,000` base units (`1.0 TURB3`).

- **Command:**
  ```bash
  npm run spl:mint
  ```
- **Terminal Output:**
  ```text
  Your ata is : B4uNhMYZmAqS9sDRRKL7ytg6sFkbKwqBfWM37HMQrr59
  mint txid: 3FESe6qGaHHiVefBcWCkfRLJHTLqTgLEcHhePiYkPoCC7CJry2W9k34Xprbg5xLTqMaxrjV8AcdyMUY8wi4GDyn2
  ```
- **Solana Explorer:** [View Mint-To Transaction](https://explorer.solana.com/tx/3FESe6qGaHHiVefBcWCkfRLJHTLqTgLEcHhePiYkPoCC7CJry2W9k34Xprbg5xLTqMaxrjV8AcdyMUY8wi4GDyn2?cluster=devnet)

---

### 1.4 Transfer Tokens (ATA to ATA)
Derives the recipient ATA (`3X3MD9BFHBnGwki3XQ9GLCJSAfrnSewzQTrf4MKrmg9V`), creates it if needed, and transfers `1,000,000` base units (`1.0 TURB3`) via `transferChecked`.

- **Command:**
  ```bash
  npm run spl:transfer
  ```
- **Terminal Output:**
  ```text
  Your fromAta is : B4uNhMYZmAqS9sDRRKL7ytg6sFkbKwqBfWM37HMQrr59
  Your toAta is : Wv6mv8BhBf1gBLj42QfuACYUCEzpriCy2ZcLEyhDP6B
  mint txid: 22QRaYPcVLU1mFANgQ2ZP5zior4RCTkKZuoreiPQVgdBN6g1ZpUHtJ3UGZjkXuoJ4sgNeYv3epELxVbE9C16mV1d
  ```
- **Solana Explorer:** [View Token Transfer Transaction](https://explorer.solana.com/tx/22QRaYPcVLU1mFANgQ2ZP5zior4RCTkKZuoreiPQVgdBN6g1ZpUHtJ3UGZjkXuoJ4sgNeYv3epELxVbE9C16mV1d?cluster=devnet)

---

### 📊 SPL Token Summary Table

| Property | Value / Address | Explorer Link |
|---|---|---|
| **Mint Address** | `4xaTz8bEzFaYE1eXpaVS5R1GVAafZxiCXt6Phnu7Ka3r` | [Explorer](https://explorer.solana.com/address/4xaTz8bEzFaYE1eXpaVS5R1GVAafZxiCXt6Phnu7Ka3r?cluster=devnet) |
| **Token Name** | `Turbin3 Sohum` | - |
| **Token Symbol** | `TURB3` | - |
| **Decimals** | `6` | - |
| **Sender ATA** | `B4uNhMYZmAqS9sDRRKL7ytg6sFkbKwqBfWM37HMQrr59` | [Explorer](https://explorer.solana.com/address/B4uNhMYZmAqS9sDRRKL7ytg6sFkbKwqBfWM37HMQrr59?cluster=devnet) |
| **Recipient ATA** | `Wv6mv8BhBf1gBLj42QfuACYUCEzpriCy2ZcLEyhDP6B` | [Explorer](https://explorer.solana.com/address/Wv6mv8BhBf1gBLj42QfuACYUCEzpriCy2ZcLEyhDP6B?cluster=devnet) |
| **Recipient Owner** | `3X3MD9BFHBnGwki3XQ9GLCJSAfrnSewzQTrf4MKrmg9V` | [Explorer](https://explorer.solana.com/address/3X3MD9BFHBnGwki3XQ9GLCJSAfrnSewzQTrf4MKrmg9V?cluster=devnet) |

---

## 🎨 Task 2: Mint an NFT Using Metaplex Core (MPL-Core)

Uses Metaplex's next-generation **MPL Core** standard for single-account, cost-effective NFTs on Solana.

### 2.1 Upload Image to Irys Decentralized Storage
Uploads `generug.png` to Irys Devnet storage.

- **Command:**
  ```bash
  npm run nft:image
  ```
- **Terminal Output:**
  ```text
  Your image URI:  https://gateway.irys.xyz/ChuxR64wx8qjnAQf18ZiUmvhFaxqdg4xrYZYCypes3vz
  ```
- **Gateway Link:** [View Image on Irys Gateway](https://gateway.irys.xyz/ChuxR64wx8qjnAQf18ZiUmvhFaxqdg4xrYZYCypes3vz)

---

### 2.2 Upload NFT Metadata JSON
Uploads the standardized Metaplex Core JSON metadata referencing the uploaded image.

- **Command:**
  ```bash
  npm run nft:metadata
  ```
- **Terminal Output:**
  ```text
  metadata uri: https://gateway.irys.xyz/DQMA7LAWABboLnfBQNgimsKkEgxm54aHCA9hPNfpCNoC
  ```
- **Gateway Link:** [View Metadata JSON on Irys Gateway](https://gateway.irys.xyz/DQMA7LAWABboLnfBQNgimsKkEgxm54aHCA9hPNfpCNoC)

---

### 2.3 Mint Metaplex Core NFT
Mints the NFT asset with the initial name `SohumRug` pointing to the Irys metadata URI.

- **Command:**
  ```bash
  npm run nft:mint
  ```
- **Terminal Output:**
  ```text
  signature 3JYHS8nfCr6UBPR5ifgcGDi3hAeM2Ryztq9oGAoEVidvCWmn71GDVuNyu2wcwGrFacmtVLbmuF62scMFLH7HptZ6 , asset : 3UPbs9SRNYMwRMm2r5B4EFAJx1YseP2WUrVGoeqP5MFR
  ```
- **Solana Explorer:** [View NFT Mint Transaction](https://explorer.solana.com/tx/3JYHS8nfCr6UBPR5ifgcGDi3hAeM2Ryztq9oGAoEVidvCWmn71GDVuNyu2wcwGrFacmtVLbmuF62scMFLH7HptZ6?cluster=devnet)
- **Asset Address:** [3UPbs9SRNYMwRMm2r5B4EFAJx1YseP2WUrVGoeqP5MFR](https://explorer.solana.com/address/3UPbs9SRNYMwRMm2r5B4EFAJx1YseP2WUrVGoeqP5MFR?cluster=devnet)

---

## 🔄 Task 3: Update NFT Name & Metadata (Update Authority)

As the update authority, fetches the existing asset from chain, uploads new updated metadata to Irys (`Sohum Rug 2`), and updates the on-chain asset name and URI using Metaplex Core `update`.

- **Command:**
  ```bash
  npm run nft:update
  ```
- **Terminal Output:**
  ```text
  new metadata uri: https://gateway.irys.xyz/8EBNB6W7W64uTbKx31efXUjb5K6KZ2npVH6LMMRGShhL 
  Successfully updated NFT!
  signature 2jgzTJyg3jMPsmpng3t8VjZX13eRcjUqtCzNZjcqS9EFKUJd72dAZetzdjQTnSpEF7q5wdwYjWZqFdKkWiw1SLup , asset : 3UPbs9SRNYMwRMm2r5B4EFAJx1YseP2WUrVGoeqP5MFR
  ```
- **New Metadata Gateway:** [View Updated Metadata on Irys](https://gateway.irys.xyz/8EBNB6W7W64uTbKx31efXUjb5K6KZ2npVH6LMMRGShhL)
- **Solana Explorer:** [View NFT Update Transaction](https://explorer.solana.com/tx/2jgzTJyg3jMPsmpng3t8VjZX13eRcjUqtCzNZjcqS9EFKUJd72dAZetzdjQTnSpEF7q5wdwYjWZqFdKkWiw1SLup?cluster=devnet)

---

### 📊 MPL Core NFT Summary Table

| Field | Initial State | Updated State |
|---|---|---|
| **Asset Address** | `3UPbs9SRNYMwRMm2r5B4EFAJx1YseP2WUrVGoeqP5MFR` | `3UPbs9SRNYMwRMm2r5B4EFAJx1YseP2WUrVGoeqP5MFR` |
| **Name** | `SohumRug` | `Updated NFT Name` (`Sohum Rug 2`) |
| **Description** | `Generug NFT` | `Generug NFT Updated` |
| **Image URI** | `https://gateway.irys.xyz/ChuxR64wx8qjnAQf18ZiUmvhFaxqdg4xrYZYCypes3vz` | `https://gateway.irys.xyz/ChuxR64wx8qjnAQf18ZiUmvhFaxqdg4xrYZYCypes3vz` |
| **Metadata URI** | [DQMA7LAW...](https://gateway.irys.xyz/DQMA7LAWABboLnfBQNgimsKkEgxm54aHCA9hPNfpCNoC) | [8EBNB6W7...](https://gateway.irys.xyz/8EBNB6W7W64uTbKx31efXUjb5K6KZ2npVH6LMMRGShhL) |
| **Transaction** | [Mint Tx](https://explorer.solana.com/tx/3JYHS8nfCr6UBPR5ifgcGDi3hAeM2Ryztq9oGAoEVidvCWmn71GDVuNyu2wcwGrFacmtVLbmuF62scMFLH7HptZ6?cluster=devnet) | [Update Tx](https://explorer.solana.com/tx/2jgzTJyg3jMPsmpng3t8VjZX13eRcjUqtCzNZjcqS9EFKUJd72dAZetzdjQTnSpEF7q5wdwYjWZqFdKkWiw1SLup?cluster=devnet) |

---

## 📸 Terminal Execution Logs (All Scripts Passing)

### 1. SPL Token Lifecycle Execution
```console
$ npm run spl:init
Sending transaction and waiting for confirmation...
mint address: 4xaTz8bEzFaYE1eXpaVS5R1GVAafZxiCXt6Phnu7Ka3r. Transaction Signature: 2PknoC661jNHZR9zQtCuVUY8R5iSxLCVbM1aroWpL9ZJCGUFCdJsbzxesD9VTXe6yXcBaKWAmmt1cZD1gwJkweY8

$ npm run spl:metadata
signature:  3pukoPCpzMqGXcFmMSsH5tZPu9BF1ocaDKMyE7Q2Pqn5phQSbh7syAfD52pTZLyJGzq2r5Zz4XNvB96J2cVKvCdg

$ npm run spl:mint
Your ata is : B4uNhMYZmAqS9sDRRKL7ytg6sFkbKwqBfWM37HMQrr59
mint txid: 3FESe6qGaHHiVefBcWCkfRLJHTLqTgLEcHhePiYkPoCC7CJry2W9k34Xprbg5xLTqMaxrjV8AcdyMUY8wi4GDyn2

$ npm run spl:transfer
Your fromAta is : B4uNhMYZmAqS9sDRRKL7ytg6sFkbKwqBfWM37HMQrr59
Your toAta is : Wv6mv8BhBf1gBLj42QfuACYUCEzpriCy2ZcLEyhDP6B
mint txid: 22QRaYPcVLU1mFANgQ2ZP5zior4RCTkKZuoreiPQVgdBN6g1ZpUHtJ3UGZjkXuoJ4sgNeYv3epELxVbE9C16mV1d
```

### 2. Metaplex Core NFT Mint & Update Execution
```console
$ npm run nft:image
Your image URI:  https://gateway.irys.xyz/ChuxR64wx8qjnAQf18ZiUmvhFaxqdg4xrYZYCypes3vz

$ npm run nft:metadata
metadata uri: https://gateway.irys.xyz/DQMA7LAWABboLnfBQNgimsKkEgxm54aHCA9hPNfpCNoC 

$ npm run nft:mint
signature 3JYHS8nfCr6UBPR5ifgcGDi3hAeM2Ryztq9oGAoEVidvCWmn71GDVuNyu2wcwGrFacmtVLbmuF62scMFLH7HptZ6 , asset : 3UPbs9SRNYMwRMm2r5B4EFAJx1YseP2WUrVGoeqP5MFR

$ npm run nft:update
new metadata uri: https://gateway.irys.xyz/8EBNB6W7W64uTbKx31efXUjb5K6KZ2npVH6LMMRGShhL 
Successfully updated NFT!
signature 2jgzTJyg3jMPsmpng3t8VjZX13eRcjUqtCzNZjcqS9EFKUJd72dAZetzdjQTnSpEF7q5wdwYjWZqFdKkWiw1SLup , asset : 3UPbs9SRNYMwRMm2r5B4EFAJx1YseP2WUrVGoeqP5MFR
```

---

## 🔗 Reference Documentation
- [Solana Token Documentation](https://solana.com/docs/tokens)
- [Solana Kit (@solana/kit)](https://www.solanakit.com/)
- [Metaplex Core (MPL-Core)](https://developers.metaplex.com/core)
- [Metaplex UMI Framework](https://github.com/metaplex-foundation/umi)
- [Irys Decentralized Storage](https://docs.irys.xyz/)
