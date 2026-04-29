---
name: EigenCloud
description: Restake LSTs on EigenLayer to earn AVS operator yield — stake, delegate, and manage your restaking positions
version: "0.1.1"
---

# EigenCloud

Restake liquid staking tokens (LSTs) on EigenLayer to earn additional yield from AVS operators. Supports 11 tokens including stETH, rETH, cbETH, and EIGEN on Ethereum mainnet.

## Data Trust Boundary

**All on-chain data is read directly via eth_call — no third-party indexers.** Position queries call `StrategyManager.getDeposits()` and `DelegationManager.delegatedTo()` directly on Ethereum mainnet. Strategy and token addresses are hardcoded from verified on-chain contract storage.

**Untrusted inputs**: `--operator` address (validated as 42-char hex before use); `--token` symbol (validated against hardcoded strategy list); `--amount` (parsed and validated locally before calldata construction).

---

## Proactive Onboarding

When a user signals they are **new or just installed** this plugin — e.g. "I just installed eigencloud", "how do I get started with EigenLayer restaking", "what can I do with this" — **do not wait for them to ask specific questions.** Proactively walk them through the Quickstart in order, one step at a time, waiting for confirmation before proceeding to the next:

1. **Check wallet** — run `onchainos wallet addresses --chain 1`. If no Ethereum address, direct them to connect via `onchainos wallet login`. EigenLayer is mainnet-only — no testnet support.
2. **Check balance** — run `onchainos wallet balance --chain 1`. They need an LST (stETH, rETH, cbETH, etc.) to restake. If they only have ETH, explain they need to acquire an LST first (e.g. via Lido for stETH).
3. **Show supported tokens** — run `eigencloud strategies` to show all 11 supported LSTs and their strategy addresses.
4. **Check existing positions** — run `eigencloud positions` to see if they already have restaked shares or a delegation.
5. **Preview before staking** — run `eigencloud stake --token stETH --amount 0.01` (no `--confirm`) to show the preview. Explain the two-step flow (approve + deposit).
6. **Execute** — once they confirm, re-run with `--confirm`.
7. **Delegate** — after staking, suggest delegating to an operator: `eigencloud delegate --operator <address>` to start earning AVS rewards.

Do not dump all steps at once. Guide conversationally — confirm each step before moving on.

---

## Quickstart

New to EigenCloud? Follow these steps to go from zero to your first restaked position.

### Step 1 — Connect your wallet

```bash
onchainos wallet login your@email.com
onchainos wallet addresses --chain 1
```

EigenLayer operates on **Ethereum mainnet only** (chain ID 1).

### Step 2 — Check your LST balance

```bash
onchainos wallet balance --chain 1
```

You need a liquid staking token (stETH, rETH, cbETH, etc.) to restake. Run `eigencloud strategies` to see all supported tokens. Minimum: any non-zero amount (no enforced minimum on-chain).

### Step 3 — View supported strategies

```bash
eigencloud strategies
```

Shows all 11 supported LSTs with their token addresses, strategy contracts, and decimals.

### Step 4 — Check your current positions

```bash
eigencloud positions
# Or check another wallet:
eigencloud positions --wallet 0xYourAddress
```

### Step 5 — Preview before staking (no --confirm = safe preview)

```bash
# Preview (no on-chain action):
eigencloud stake --token stETH --amount 0.01

# Execute (adds approve + depositIntoStrategy txs):
eigencloud stake --token stETH --amount 0.01 --confirm
```

Staking sends **two transactions**: an ERC-20 `approve` and then `depositIntoStrategy`. The binary waits 15s for the approval to confirm before sending the deposit.

### Step 6 — Delegate to an AVS operator

```bash
# Preview:
eigencloud delegate --operator 0xOperatorAddress

# Execute:
eigencloud delegate --operator 0xOperatorAddress --confirm
```

Delegation applies to **all current and future** restaked positions. Find operators at `app.eigenlayer.xyz/operator`.

### Step 7 — Check positions after staking

```bash
eigencloud positions
```

Expected output: `positions` array with `symbol`, `shares`, and `delegated: true` once delegated.

---

## Overview

EigenLayer is a restaking protocol on Ethereum mainnet. Restaking means depositing LSTs into EigenLayer's StrategyManager, which makes your stake available to secure Actively Validated Services (AVSs). Operators run AVS software and earn fees; by delegating to an operator, restakers share in those rewards.

**Lifecycle:**
1. Hold an LST (stETH, rETH, cbETH, etc.)
2. `stake` → approve + depositIntoStrategy (shares credited immediately)
3. `delegate` → assign shares to an operator (start earning AVS rewards)
4. `undelegate` → queue withdrawal (7-day delay, must then complete via EigenLayer app)

---

## Commands

### `eigencloud strategies`

List all supported LST strategies with their on-chain addresses.

```bash
eigencloud strategies
```

**Output fields:**

| Field | Description |
|-------|-------------|
| `symbol` | Token symbol (e.g. `stETH`) |
| `description` | Human-readable name |
| `token` | ERC-20 token contract address |
| `strategy` | EigenLayer strategy contract address |
| `decimals` | Token decimals (18 for most) |

---

### `eigencloud positions`

Show your current restaked shares and delegation status.

```bash
eigencloud positions
eigencloud positions --wallet 0xAddress   # Query another wallet
```

**Flags:**

| Flag | Description |
|------|-------------|
| `--wallet` | Address to query (defaults to active onchainos wallet) |

**Output fields:**

| Field | Description |
|-------|-------------|
| `wallet` | Address queried |
| `positions` | Array of restaked positions |
| `positions[].symbol` | Token symbol |
| `positions[].strategy` | Strategy contract address |
| `positions[].shares` | Human-readable share balance |
| `positions[].shares_raw` | Raw uint256 share balance |
| `delegated` | Whether the wallet has delegated |
| `operator` | Delegated operator address (or `"none"`) |

---

### `eigencloud stake`

Restake an LST into EigenLayer. Sends two transactions: `approve` + `depositIntoStrategy`.

```bash
# Preview (safe — no tx sent):
eigencloud stake --token stETH --amount 0.01

# Execute:
eigencloud stake --token stETH --amount 0.01 --confirm

# Dry-run (build calldata only, no onchainos):
eigencloud stake --token stETH --amount 0.01 --dry-run
```

**Flags:**

| Flag | Description |
|------|-------------|
| `--token` | LST symbol (e.g. `stETH`, `rETH`, `cbETH`) — required |
| `--amount` | Amount to restake in human-readable form (e.g. `0.01`) — required |
| `--confirm` | Broadcast both transactions |
| `--dry-run` | Build calldata without calling onchainos (conflicts with `--confirm`) |

**Execution modes:**

| Mode | Command | Effect |
|------|---------|--------|
| Preview | `eigencloud stake --token X --amount Y` | Shows preview JSON, no on-chain action |
| Execute | `eigencloud stake --token X --amount Y --confirm` | Sends approve + depositIntoStrategy |
| Dry-run | `eigencloud stake --token X --amount Y --dry-run` | Builds calldata only |

**Output (confirmed):**

```json
{
  "ok": true,
  "action": "stake",
  "token": "stETH",
  "amount": "0.01",
  "wallet": "0x...",
  "strategy": "0x93c4b944D1e2B80D4A8d6A1af43cC64cFB2E4A04",
  "txs": [
    {"step": "approve",             "tx_hash": "0x..."},
    {"step": "depositIntoStrategy", "tx_hash": "0x..."}
  ]
}
```

---

### `eigencloud delegate`

Delegate all restaked shares to an EigenLayer operator.

```bash
# Preview:
eigencloud delegate --operator 0xOperatorAddress

# Execute:
eigencloud delegate --operator 0xOperatorAddress --confirm

# Dry-run:
eigencloud delegate --operator 0xOperatorAddress --dry-run
```

**Flags:**

| Flag | Description |
|------|-------------|
| `--operator` | Operator address (42-char hex, required) |
| `--confirm` | Broadcast the delegateTo transaction |
| `--dry-run` | Build calldata without calling onchainos |

**Note:** Delegation applies to all current and future restaked positions. Only one operator can be delegated to at a time — undelegating is required before re-delegating. Works with public operators (no approver); operators requiring approval signatures are not supported.

**Output (confirmed):**

```json
{
  "ok": true,
  "action": "delegate",
  "operator": "0x...",
  "wallet": "0x...",
  "tx_hash": "0x..."
}
```

---

### `eigencloud undelegate`

Undelegate from the current operator. Queues **all** restaked shares for withdrawal.

```bash
# Preview:
eigencloud undelegate

# Execute:
eigencloud undelegate --confirm

# Dry-run:
eigencloud undelegate --dry-run
```

**Flags:**

| Flag | Description |
|------|-------------|
| `--confirm` | Broadcast the undelegate transaction |
| `--dry-run` | Build calldata without calling onchainos |

**Warning:** Undelegating queues ALL restaked positions for withdrawal with a **7-day delay**. After the delay, each position must be completed separately via the EigenLayer app at `app.eigenlayer.xyz`.

**Output (confirmed):**

```json
{
  "ok": true,
  "action": "undelegate",
  "wallet": "0x...",
  "tx_hash": "0x...",
  "next_step": "After 7 days, complete your withdrawal via the EigenLayer app at app.eigenlayer.xyz"
}
```

---

## Supported Tokens

| Symbol | Description | Decimals |
|--------|-------------|----------|
| `stETH` | Lido Staked ETH | 18 |
| `rETH` | Rocket Pool ETH | 18 |
| `cbETH` | Coinbase Wrapped Staked ETH | 18 |
| `mETH` | Mantle Staked ETH | 18 |
| `swETH` | Swell ETH | 18 |
| `wBETH` | Wrapped Binance Beacon ETH | 18 |
| `sfrxETH` | Staked Frax ETH | 18 |
| `osETH` | StakeWise Staked ETH | 18 |
| `ETHx` | Stader ETHx | 18 |
| `ankrETH` | Ankr Staked ETH | 18 |
| `EIGEN` | EigenLayer Token | 18 |

Run `eigencloud strategies` for full addresses.

---

## Contracts

| Contract | Address | Chain |
|----------|---------|-------|
| StrategyManager | `0x858646372CC42E1A627fcE94aa7A7033e7CF075A` | Ethereum (1) |
| DelegationManager | `0x39053D51B77DC0d36036Fc1fCc8Cb819df8Ef37A` | Ethereum (1) |

---

## Install

```bash
npx skills add okx/plugin-store --skill eigencloud-plugin
eigencloud --version   # Expected: 0.1.1
```
