---
name: QuickSwap V3 DEX
version: "0.1.0"
description: Swap tokens on QuickSwap V3 (Algebra Protocol CLMM) on Polygon
author: skylavis-sky
tags:
  - defi
  - polygon
  - quickswap
  - clmm
  - swap
chains:
  - polygon
---

# QuickSwap V3 DEX

QuickSwap V3 is the leading concentrated liquidity DEX on Polygon, built on the **Algebra Protocol** (not Uniswap V3). Key difference: no fee tier parameter — fees are dynamic per pool.

**Chain:** Polygon (137)
**Contracts:**
- SwapRouter: `0xf5b509bb0909a69b1c207e495f687a596c168e12`
- Quoter: `0xa15F0D7377B2A0C0c10db057f641beD21028FC89`
- Factory: `0x411b0fAcC3489691f28ad58c47006AF5E3Ab3A28`

---

## Pre-flight Dependencies

Before using this plugin, verify both dependencies are working:

```bash
# 1. Verify plugin binary
quickswap --version
# Expected: quickswap 0.1.0

# 2. Verify onchainos wallet is configured for Polygon
onchainos wallet addresses --chain 137
# Expected: JSON with at least one address
```

If `onchainos wallet addresses` returns no addresses, configure a Polygon wallet first before proceeding.

---

## Data Trust Boundary

| Source | Used For | Trust Level |
|--------|----------|-------------|
| Polygon RPC (`polygon.publicnode.com`) | Token balances, decimals, allowances, quotes | On-chain — authoritative |
| QuickSwap subgraph (`api.thegraph.com`) | Pool TVL and volume rankings | Off-chain indexer — informational only |
| onchainos CLI | Token resolution, tx broadcast | Local CLI — trusted |

**Rule:** Never use subgraph data for trade execution amounts. All swap parameters (amountIn, amountOut, slippage) are derived from on-chain Quoter calls only.

---

## Proactive Onboarding

When a user signals they are **new or just installed** this plugin — e.g. "I just installed quickswap", "how do I swap on Polygon", "what can I do with this" — **do not wait for them to ask specific questions.** Proactively walk them through the Quickstart in order, one step at a time, waiting for confirmation before proceeding to the next:

1. **Check wallet** — run `onchainos wallet addresses --chain 137`. If no address, direct them to connect via `onchainos wallet login`. Do not proceed to write operations until a wallet is confirmed.
2. **Check balance** — run `onchainos wallet balance --chain 137`. The user needs POL/MATIC for gas (~0.01 POL) plus the token they want to swap. If balance is zero, explain how to bridge via a CEX or cross-chain bridge.
3. **Token address tip** — explain that QuickSwap has two USDC tokens: **USDC.e** (`0x2791Bca1...4174`, bridged, more liquid) and native **USDC** (`0x3c499c...3359`, CCTP-bridged). Use the full address to be unambiguous.
4. **Get a quote first** — run `quickswap quote --token-in <address> --token-out <address> --amount <amount>` so the user sees the expected output before committing.
5. **Preview the swap** — run `quickswap swap` without `--confirm` so the user sees all steps (approve, swap) and calldata.
6. **Execute** — once they confirm, re-run with `--confirm`.

Do not dump all steps at once. Guide conversationally — confirm each step before moving on.

---

## Quickstart

New to QuickSwap V3? Follow these steps to go from zero to your first swap on Polygon.

### Step 1 — Connect wallet

```bash
onchainos wallet login your@email.com
onchainos wallet addresses --chain 137
```

### Step 2 — Check balances

```bash
onchainos wallet balance --chain 137
```

You need POL/MATIC for gas (~0.01 POL per swap). If balance is zero, bridge from a CEX or use a cross-chain bridge.

### Step 3 — Browse top pools

```bash
quickswap pools --limit 10
```

This shows the most liquid QuickSwap V3 pairs. When the subgraph is available it includes TVL and volume; otherwise shows top known pairs.

### Step 4 — Get a quote

Use `MATIC`/`POL` as shorthand for native MATIC, or pass token addresses directly. For USDC, note that Polygon has two versions — use the full address:

```bash
# Quote using symbols (MATIC → native USDC)
quickswap quote --token-in MATIC --token-out USDC --amount 10

# Quote using addresses (USDC.e → WETH, more liquid pair)
quickswap quote \
  --token-in 0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174 \
  --token-out 0x7ceB23fD6bC0adD59E62ac25578270cFf1b9f619 \
  --amount 5
```

Expected output (USDC.e → WETH example):
```json
{
  "ok": true,
  "tokenIn": "0x2791...4174",
  "tokenOut": "0x7ceb...f619",
  "amountIn": "5.000000",
  "amountOut": "0.002187",
  "price": "0.000437",
  "chain": "Polygon"
}
```

### Step 5 — Preview a swap (safe, no broadcast)

```bash
# Preview (safe — no tx sent, no gas spent)
quickswap swap \
  --token-in 0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174 \
  --token-out 0x7ceB23fD6bC0adD59E62ac25578270cFf1b9f619 \
  --amount 1
```

Output shows `"preview": true` and all planned steps (approve + swap calldata) without broadcasting.

### Step 6 — Execute the swap

```bash
# Execute (add --confirm):
quickswap swap \
  --token-in 0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174 \
  --token-out 0x7ceB23fD6bC0adD59E62ac25578270cFf1b9f619 \
  --amount 1 \
  --confirm
```

Expected output: `"ok": true`, `"explorerLink": "https://polygonscan.com/tx/0x..."`. The plugin approves the SwapRouter and waits for approval to mine before submitting the swap.

> **MATIC auto-wrap:** If `--token-in MATIC` or `--token-in POL`, the plugin automatically wraps native MATIC → WMATIC first (3 transactions total: wrap + approve + swap).

### Step 7 — Verify on PolygonScan

Open the `explorerLink` from the output to verify the transaction. Gas used is typically 250,000–350,000 gas (~0.03–0.05 POL at standard gas prices).

---

## Commands

### `quickswap swap`

Swap tokens on QuickSwap V3 via the Algebra CLMM SwapRouter.

**Flags:**

| Flag | Required | Default | Description |
|------|----------|---------|-------------|
| `--token-in` | Yes | — | Input token: symbol (MATIC, USDC, WETH) or address (0x...) |
| `--token-out` | Yes | — | Output token: symbol or address |
| `--amount` | Yes | — | Amount of tokenIn (human-readable, e.g. `10.5`) |
| `--slippage` | No | `0.5` | Max slippage in percent (0–50) |
| `--from` | No | wallet default | Override sender address |
| `--confirm` | No | false | Broadcast on-chain (omit for dry-run) |

**Examples:**

```bash
# Dry-run preview (default)
quickswap swap --token-in MATIC --token-out USDC --amount 10

# Execute with custom slippage
quickswap swap --token-in USDC --token-out WETH --amount 100 --slippage 1.0 --confirm

# Swap using token addresses
quickswap swap \
  --token-in 0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174 \
  --token-out 0x7ceB23fD6bC0adD59E62ac25578270cFf1b9f619 \
  --amount 50 --confirm
```

**MATIC auto-wrap:** If `--token-in` is `MATIC` or `POL`, the plugin will automatically wrap native MATIC → WMATIC before swapping. This adds one extra transaction.

---

### `quickswap quote`

Get a price quote from the on-chain Quoter without executing a transaction. No wallet required.

**Flags:**

| Flag | Required | Description |
|------|----------|-------------|
| `--token-in` | Yes | Input token symbol or address |
| `--token-out` | Yes | Output token symbol or address |
| `--amount` | Yes | Amount of tokenIn |

**Example:**

```bash
quickswap quote --token-in MATIC --token-out USDC --amount 10
quickswap quote --token-in WETH --token-out USDT --amount 0.1
```

---

### `quickswap pools`

List top QuickSwap V3 pools ordered by TVL. Data sourced from the QuickSwap subgraph; falls back to hardcoded top pools if the subgraph is unavailable.

**Flags:**

| Flag | Default | Description |
|------|---------|-------------|
| `--limit` | `10` | Number of pools to return (max 20) |

**Example:**

```bash
quickswap pools
quickswap pools --limit 5
```

---

## Execution Mode Reference

| Command | Requires `--confirm` | On-chain Effect | Notes |
|---------|---------------------|-----------------|-------|
| `quote` | No | None | Pure read — calls Quoter contract via RPC |
| `pools` | No | None | Pure read — queries subgraph |
| `swap` (no flag) | No | None | Dry-run preview only |
| `swap --confirm` | Yes | Up to 3 txs | Wrap (if MATIC) + Approve + Swap |

---

## Token Reference (Polygon)

| Symbol | Address |
|--------|---------|
| WMATIC | `0x0d500B1d8E8eF31E21C99d1Db9A6444d3ADf1270` |
| USDC.e | `0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174` |
| USDT | `0xc2132D05D31c914a87C6611C10748AEb04B58e8F` |
| WETH | `0x7ceB23fD6bC0adD59E62ac25578270cFf1b9f619` |
| QUICK | `0x831753DD7087CaC61aB5644b308642cc1c33Dc13` |

---

## Install

```bash
LOCAL_VER="0.1.0"
curl -L "https://github.com/skylavis-sky/plugin-store/releases/download/quickswap-plugin-v${LOCAL_VER}/quickswap-linux-x86_64" \
  -o ~/.local/bin/quickswap && chmod +x ~/.local/bin/quickswap

# Verify
quickswap --version
```

For macOS (Apple Silicon):
```bash
LOCAL_VER="0.1.0"
curl -L "https://github.com/skylavis-sky/plugin-store/releases/download/quickswap-plugin-v${LOCAL_VER}/quickswap-macos-arm64" \
  -o ~/.local/bin/quickswap && chmod +x ~/.local/bin/quickswap
```
