---
name: Fluid
description: Supply collateral and borrow on Fluid Protocol's smart lending vaults on Ethereum and Arbitrum
version: "0.1.1"
---

# Fluid

Fluid Protocol is Instadapp's unified lending and borrowing layer. Positions are managed as ERC-721 NFTs through a single `operate()` entry point on each vault. Supported on Ethereum mainnet (chain 1) and Arbitrum (chain 42161).

---

## Data Trust Boundary

All on-chain reads (vault list, token symbols, position data) come directly from Fluid's on-chain resolvers via public RPC. No off-chain price oracles or third-party APIs are involved in the read path. Token symbols are resolved from `symbol()` ERC-20 calls on-chain; hardcoded fallbacks are used only when the RPC returns no data.

Write operations are constructed locally and signed via onchainos — no transaction data is ever routed through a third-party API. All amounts shown in previews match the calldata exactly.

---

## Proactive Onboarding

When a user signals they are **new or just installed** this plugin — e.g. "I just installed fluid", "how do I get started", "what can I do with this" — **do not wait for them to ask specific questions.** Proactively walk them through the Quickstart in order, one step at a time, waiting for confirmation before proceeding to the next:

1. **Check wallet** — run `onchainos wallet addresses --chain 1`. If no address, direct them to connect via `onchainos wallet login`. Do not proceed to write operations until a wallet is confirmed.
2. **Check balance** — run `onchainos wallet balance --chain 1`. For ETH vaults, a minimum of ~0.05 ETH is needed. For stablecoin debt, the wallet needs the debt token to repay.
3. **Browse vaults** — run `fluid vaults` to show what's available. Ask what pair they're interested in. Show both ETH and Arbitrum if relevant.
4. **Preview the supply** — run `fluid supply --vault <addr> --amount <n>` without `--confirm` so they see the calldata and approval flow before any on-chain action.
5. **Execute supply** — once they confirm, re-run with `--confirm`.
6. **Preview borrow** — after supply confirms, show `fluid borrow --vault <addr> --nft-id <id> --amount <n>` preview.
7. **Execute borrow** — re-run with `--confirm`.

Do not dump all steps at once. Guide conversationally — confirm each step before moving on.

---

## Quickstart

New to Fluid? Follow these steps to go from zero to your first lending position.

### Step 1 — Connect your wallet

```bash
onchainos wallet login your@email.com
onchainos wallet addresses --chain 1
```

### Step 2 — Check your balance

```bash
onchainos wallet balance --chain 1
```

Minimum recommended: 0.05 ETH for an ETH/USDC vault. If zero, bridge or transfer ETH to Ethereum mainnet.

### Step 3 — Browse available vaults

```bash
# Ethereum mainnet (T1 vaults only)
fluid vaults

# Arbitrum
fluid vaults --chain 42161

# Show all vault types including smart-collateral/smart-debt vaults
fluid vaults --all
```

Look for the `pair` field to find the collateral/debt pair you want (e.g. `ETH/USDC`). Copy the `vault` address for subsequent commands.

### Step 4 — Preview before executing

All write commands show a safe preview by default — no on-chain action until you add `--confirm`:

```bash
# Preview (safe — no tx sent):
fluid supply --vault 0xeabb... --amount 0.1

# Execute:
fluid supply --vault 0xeabb... --amount 0.1 --confirm
```

### Step 5 — Supply collateral

```bash
fluid supply \
  --vault 0xeabbfca72f8a8bf14c4ac59e69ecb2eb69f0811c \
  --amount 0.1 \
  --confirm
```

Expected output: `"ok": true`, `"tx_hash": "0x..."`. The `nft_id` in the output is your new position ID — save it for borrow/repay/withdraw.

### Step 6 — Borrow

Before borrowing, preview the command to see the current `borrow_rate` (sourced live from the vault):

```bash
# Preview — shows borrow_rate before any on-chain action:
fluid borrow \
  --vault 0xeabbfca72f8a8bf14c4ac59e69ecb2eb69f0811c \
  --nft-id <YOUR_NFT_ID> \
  --amount 50
```

Once you've seen the rate and confirmed the amount, add `--confirm`:

```bash
fluid borrow \
  --vault 0xeabbfca72f8a8bf14c4ac59e69ecb2eb69f0811c \
  --nft-id <YOUR_NFT_ID> \
  --amount 50 \
  --confirm
```

### Step 7 — Check your positions

```bash
fluid positions
```

---

## Overview

Fluid Protocol uses a smart-collateral architecture where each vault has a collateral token and a debt token. Positions are represented as ERC-721 NFTs:

- **T1 vaults**: Standard collateral/debt pairs (e.g. ETH/USDC, wstETH/USDT)
- **T2 vaults**: One side is "smart" (auto-compounding yield token)
- **T3 vaults**: Both sides are smart

The default `vaults` command shows T1 only. Use `--all` to include T2 and T3.

All position writes go through the vault's `operate(nftId, newCol, newDebt, to)` function:
- Positive `newCol` = supply collateral
- Negative `newCol` = withdraw collateral
- Positive `newDebt` = borrow
- Negative `newDebt` = repay

---

## Commands

### `vaults` — List available vaults

```bash
fluid vaults [--chain <ID>] [--all] [--limit <N>]
```

| Flag | Default | Description |
|------|---------|-------------|
| `--chain` | `1` | Chain ID (1 = Ethereum, 42161 = Arbitrum) |
| `--all` | off | Show all vault types (default: T1 only) |
| `--limit` | `30` | Maximum vaults to display |

**Example output (T1 only, default):**
```json
{
  "chain": 1,
  "chain_name": "Ethereum",
  "filter": "T1 only",
  "showing": 30,
  "total_vaults": 164,
  "vaults": [
    {
      "vault": "0xeabbfca72f8a8bf14c4ac59e69ecb2eb69f0811c",
      "pair": "ETH/USDC",
      "col_token": "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
      "debt_token": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
      "type": "T1"
    }
  ]
}
```

**Example output (`--all`, truncated):**

When `--all` is set and the result is truncated by `--limit`, the response includes a `type_breakdown` (count per vault type across all matching vaults) and a `note` with the pagination hint:

```json
{
  "chain": 1,
  "chain_name": "Ethereum",
  "filter": "all",
  "showing": 30,
  "total_vaults": 164,
  "type_breakdown": { "T1": 62, "T2": 58, "T3": 44 },
  "note": "Showing 30 of 164 vaults. Use --limit 164 to see all.",
  "vaults": [ "..." ]
}
```

`type_breakdown` is also present (without `note`) when `--all` is set and all vaults fit within `--limit`.

---

### `positions` — Show open positions

```bash
fluid positions [--chain <ID>] [--wallet <addr>] [--limit <N>]
```

| Flag | Default | Description |
|------|---------|-------------|
| `--chain` | `1` | Chain ID |
| `--wallet` | active wallet | Address to query |
| `--limit` | `20` | Maximum positions to display |

**Example output:**
```json
{
  "wallet": "0xabc...",
  "chain": 1,
  "positions": [
    {
      "nft_id": 12345,
      "vault": "0xeabb...",
      "pair": "ETH/USDC",
      "col_symbol": "ETH",
      "col": "0.5",
      "col_raw": "500000000000000000",
      "debt_symbol": "USDC",
      "debt": "500",
      "debt_raw": "500000000"
    }
  ]
}
```

---

### `supply` — Supply collateral

```bash
fluid supply --vault <addr> --amount <n> [--nft-id <id>] [--confirm]
```

| Flag | Required | Description |
|------|----------|-------------|
| `--vault` | yes | Vault address |
| `--amount` | yes | Collateral amount (human-readable) |
| `--nft-id` | no | Existing NFT ID (0 = open new position) |
| `--chain` | no | Chain ID (default: 1) |
| `--wallet` | no | Signer address (default: active wallet) |
| `--dry-run` | no | Simulate (stub hashes, no broadcast) |
| `--confirm` | no | Required to broadcast |

**Approval flow**: For ERC-20 collateral tokens, an `approve(vault, amount)` tx fires first before the `operate()` call. For native ETH (`0xeeee...`), no approval is needed.

**Execution modes:**

| Mode | Command | On-chain? |
|------|---------|-----------|
| Preview | `fluid supply --vault ... --amount ...` | No |
| Dry-run | `fluid supply --vault ... --amount ... --dry-run` | No (stub hashes) |
| Execute | `fluid supply --vault ... --amount ... --confirm` | Yes |

---

### `borrow` — Borrow debt token

```bash
fluid borrow --vault <addr> --nft-id <id> --amount <n> [--confirm]
```

| Flag | Required | Description |
|------|----------|-------------|
| `--vault` | yes | Vault address |
| `--nft-id` | yes | Existing NFT position ID |
| `--amount` | yes | Borrow amount (human-readable) |
| `--chain` | no | Chain ID (default: 1) |
| `--wallet` | no | Signer address (default: active wallet) |
| `--dry-run` | no | Simulate (stub hashes, no broadcast) |
| `--confirm` | no | Required to broadcast |

No approval needed — debt tokens are minted/transferred out of the vault.

The preview output includes `"borrow_rate"` — the vault's current annualised borrow rate sourced on-chain from the vault's `ExchangePricesAndRates` data (1e2 precision, e.g. `"6.38%"`). This lets the agent show the rate to the user before any transaction is broadcast.

---

### `repay` — Repay debt

```bash
fluid repay --vault <addr> --nft-id <id> --amount <n> [--confirm]
```

| Flag | Required | Description |
|------|----------|-------------|
| `--vault` | yes | Vault address |
| `--nft-id` | yes | NFT position ID |
| `--amount` | yes | Repay amount (human-readable) |
| `--chain` | no | Chain ID (default: 1) |
| `--wallet` | no | Signer address (default: active wallet) |
| `--dry-run` | no | Simulate (stub hashes, no broadcast) |
| `--confirm` | no | Required to broadcast |

**Approval flow**: For ERC-20 debt tokens, an `approve(vault, amount)` tx fires first. For native ETH debt, no approval needed.

---

### `withdraw` — Withdraw collateral

```bash
fluid withdraw --vault <addr> --nft-id <id> --amount <n> [--confirm]
```

| Flag | Required | Description |
|------|----------|-------------|
| `--vault` | yes | Vault address |
| `--nft-id` | yes | NFT position ID |
| `--amount` | yes | Withdraw amount (human-readable) |
| `--chain` | no | Chain ID (default: 1) |
| `--wallet` | no | Signer address (default: active wallet) |
| `--dry-run` | no | Simulate (stub hashes, no broadcast) |
| `--confirm` | no | Required to broadcast |

No approval needed — collateral is returned directly to your wallet.

---

### `close` — Close a position atomically

Repays all outstanding debt and withdraws all collateral in a **single `operate()` call**. Use this when `repay` fails because the position is below the vault's minimum floor (Fluid error `0x60121cca` or `0xdee51a8a`).

```bash
fluid close --nft-id <id> [--chain <ID>] [--confirm]
```

| Flag | Required | Description |
|------|----------|-------------|
| `--nft-id` | yes | NFT position ID to close |
| `--chain` | no | Chain ID (default: 1) |
| `--wallet` | no | Signer address (default: active wallet) |
| `--dry-run` | no | Simulate (stub hashes, no broadcast) |
| `--confirm` | no | Required to broadcast |

**How it works**: Fetches the current `col_raw` and `debt_raw` from the PositionsResolver, then calls `operate(nftId, -col_raw, -debt_raw, wallet)`. Both sides are set simultaneously, so the vault never sees an intermediate below-floor state.

**Approval flow**: For ERC-20 debt tokens, an `approve(vault, debt_amount)` tx fires first. For native ETH debt, no approval needed.

**Example output:**
```json
{
  "ok": true,
  "action": "close",
  "vault": "0xeabb...",
  "nft_id": 12345,
  "col_symbol": "ETH",
  "col_withdrawn": "0.0001",
  "debt_symbol": "USDC",
  "debt_repaid": "0.15",
  "approve_tx_hash": "0x...",
  "tx_hash": "0x...",
  "wallet": "0xabc...",
  "chain": 42161
}
```

---

## Supported Chains

| Chain | ID | Vaults |
|-------|----|--------|
| Ethereum | 1 | ~164 |
| Arbitrum | 42161 | ~88 |

---

## Contract Addresses (same on all chains)

| Contract | Address |
|----------|---------|
| VaultResolver | `0xA5C3E16523eeeDDcC34706b0E6bE88b4c6EA95cC` |
| PositionsResolver | `0xaA21a86030EAa16546A759d2d10fd3bF9D053Bc7` |
| NFT Contract | `0x324c5dc1fc42c7a4d43d92df1eba58a54d13bf2d` |

---

## Install

```bash
# Install via plugin store
npx skills add okx/plugin-store --skill fluid-plugin --yes

# Verify
fluid --version
# Expected: fluid 0.1.1
```

### Telemetry

```bash
# Version check
curl -s -X POST "https://api.okx.com/plugin-store/v1/telemetry" \
  -H "Content-Type: application/json" \
  -d '{"plugin":"fluid-plugin","version":"0.1.1","event":"install"}'
```
