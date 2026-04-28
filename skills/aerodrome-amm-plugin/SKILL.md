---
name: Aerodrome AMM
description: Swap tokens and provide liquidity on Aerodrome AMM (volatile/stable pools) on Base
version: "0.1.0"
tools:
  - name: aerodrome-amm
    description: Swap, provide liquidity, and claim fees on Aerodrome AMM (classic xy=k and stableswap pools) on Base (chain 8453)
---

## Pre-flight Dependencies (auto-injected by Plugin Store CI)

<!-- onchainos version check injected here -->

---

## Do NOT use for...

- Concentrated liquidity (tick-range) positions → use `aerodrome-slipstream` instead
- Any chain other than Base (8453)
- Gauge staking or AERO emissions — use the Aerodrome UI for those

---

## Proactive Onboarding

When a user signals they are **new or just installed** this plugin — e.g. "I just installed aerodrome-amm", "how do I get started", "what can I do" — **do not wait for specific questions.** Walk them through the Quickstart conversationally, one step at a time:

1. **Check wallet** — run `onchainos wallet addresses --chain 8453`. If no address, direct them to `onchainos wallet login`. Do not proceed to write operations until a wallet is confirmed.
2. **Check balance** — run `onchainos wallet balance --chain 8453`. ETH (WETH) or USDC on Base is needed for swaps; both tokens needed for liquidity.
3. **Explore pools** — run `aerodrome-amm pools --token-a WETH --token-b USDC` to show what's available.
4. **Quote first** — run `aerodrome-amm quote` so the user sees the expected output before any on-chain action.
5. **Preview swap** — run `aerodrome-amm swap` without `--confirm`; show the preview JSON.
6. **Execute** — once the user confirms, re-run with `--confirm`.

Do not dump all steps at once. Guide conversationally — confirm each step before moving on.

---

## Quickstart

New to Aerodrome AMM? Follow these steps to go from zero to your first swap or LP position.

### Step 1 — Connect your wallet

```bash
onchainos wallet login your@email.com
onchainos wallet addresses --chain 8453
```

### Step 2 — Check your balance

```bash
onchainos wallet balance --chain 8453
```

You need WETH or USDC on Base (chain 8453). Minimum recommended: $5 equivalent for a first swap.

### Step 3 — Find a pool and get a quote

```bash
# Find volatile and stable WETH/USDC pools
aerodrome-amm pools --token-a WETH --token-b USDC

# Get a swap quote (auto-selects best pool)
aerodrome-amm quote --token-in WETH --token-out USDC --amount-in 0.01
```

The `quote` command shows both volatile and stable pool outputs, sorted by best return.

### Step 4 — Preview before executing

All write commands show a safe preview by default — no on-chain action until you add `--confirm`:

```bash
# Preview (safe — no tx sent):
aerodrome-amm swap --token-in WETH --token-out USDC --amount-in 0.01

# Execute (add --confirm):
aerodrome-amm swap --token-in WETH --token-out USDC --amount-in 0.01 --confirm
```

### Step 5 — Provide liquidity (optional)

```bash
# Preview adding WETH/USDC liquidity (volatile pool)
aerodrome-amm add-liquidity --token-a WETH --token-b USDC --amount-a 0.01 --amount-b 22.0

# Add --confirm to execute; add --stable for the stable pool
aerodrome-amm add-liquidity --token-a WETH --token-b USDC --amount-a 0.01 --amount-b 22.0 --confirm
```

After providing liquidity, you receive LP tokens representing your pool share. Claim accrued trading fees with `claim-fees`.

---

## Data Trust Boundary

All price and reserve data is read directly from on-chain contracts via Base RPC — no third-party price oracles. Pool addresses are resolved via `getPool()` on the Aerodrome factory (canonical source). Reserve ratios reflect current on-chain state and may differ from CEX prices — always use `quote` to get the actual swap output before confirming.

---

## Overview

Aerodrome AMM is the classic (xy=k / stableswap) liquidity layer of the Aerodrome protocol on Base. It complements Aerodrome Slipstream (concentrated liquidity) with two pool types:

- **Volatile pools** — constant-product (xy=k) AMM, suited for uncorrelated assets like WETH/USDC
- **Stable pools** — StableSwap AMM optimized for correlated/pegged assets like USDC/USDT

All commands auto-detect the best pool type unless `--stable` is passed.

### Key Contracts (Base, chain 8453)

| Contract | Address |
|----------|---------|
| Pool Factory | `0x420DD381b31aEf6683db6B902084cB0FFECe40Da` |
| Router | `0xcF77a3Ba9A5CA399B7c97c74d54e5b1Beb874E43` |

---

## Commands

### `quote` — Get swap quote (read-only)

```bash
aerodrome-amm quote --token-in WETH --token-out USDC --amount-in 0.1
aerodrome-amm quote --token-in USDC --token-out USDT --amount-in 100 --stable
```

Returns quotes from each available pool (volatile and stable), sorted by best output.

| Flag | Default | Description |
|------|---------|-------------|
| `--token-in` | required | Input token symbol or address |
| `--token-out` | required | Output token symbol or address |
| `--amount-in` | required | Human-readable amount (e.g. "0.1") |
| `--stable` | false | Only quote from stable pool |

---

### `swap` — Swap tokens

```bash
# Preview (no --confirm):
aerodrome-amm swap --token-in WETH --token-out USDC --amount-in 0.01

# Execute:
aerodrome-amm swap --token-in WETH --token-out USDC --amount-in 0.01 --confirm

# Force stable pool:
aerodrome-amm swap --token-in USDC --token-out USDT --amount-in 100 --stable --confirm
```

Auto-selects the pool giving the best output. Approves token_in to the Router if allowance is insufficient (idempotent check before approval).

| Flag | Default | Description |
|------|---------|-------------|
| `--token-in` | required | Input token |
| `--token-out` | required | Output token |
| `--amount-in` | required | Amount to swap |
| `--slippage` | `0.5` | Slippage tolerance % |
| `--stable` | false | Force stable pool |
| `--deadline-minutes` | `20` | Tx deadline |
| `--confirm` | false | Broadcast on-chain |
| `--dry-run` | false | Build calldata only |

**Preview output:**
```json
{
  "preview": true,
  "action": "swap",
  "token_in": "WETH",
  "token_out": "USDC",
  "amount_in": "0.01",
  "expected_out": "22.817474",
  "minimum_out": "22.703386",
  "slippage": "0.5%",
  "pool_type": "volatile",
  "router": "0xcF77a3Ba9A5CA399B7c97c74d54e5b1Beb874E43",
  "chain": "Base (8453)"
}
```

---

### `pools` — List pools for a token pair

```bash
aerodrome-amm pools --token-a WETH --token-b USDC
aerodrome-amm pools --token-a USDC --token-b USDT
```

Returns reserve, price, and total LP supply for volatile and stable pools on the pair.

---

### `prices` — Token price from AMM reserves

```bash
aerodrome-amm prices --token WETH
aerodrome-amm prices --token AERO --quote WETH
```

| Flag | Default | Description |
|------|---------|-------------|
| `--token` | required | Token to price |
| `--quote` | `USDC` | Quote currency |

---

### `positions` — Show LP positions

```bash
aerodrome-amm positions --token-a WETH --token-b USDC
aerodrome-amm positions --token-a USDC --token-b USDT --stable
```

Shows LP token balance, pool share %, and estimated underlying token amounts for the active wallet.

| Flag | Default | Description |
|------|---------|-------------|
| `--token-a` | required | First token of the pair |
| `--token-b` | required | Second token of the pair |
| `--stable` | false | Check stable pool only (default: checks both) |

---

### `add-liquidity` — Provide liquidity

```bash
# Preview:
aerodrome-amm add-liquidity --token-a WETH --token-b USDC --amount-a 0.01 --amount-b 22.0

# Execute:
aerodrome-amm add-liquidity --token-a WETH --token-b USDC --amount-a 0.01 --amount-b 22.0 --confirm

# Stable pool:
aerodrome-amm add-liquidity --token-a USDC --token-b USDT --amount-a 100 --amount-b 100 --stable --confirm
```

Calls `quoteAddLiquidity` first to show actual amounts used (may be adjusted to match pool ratio). Approves both tokens to Router if needed. Returns LP tokens to your wallet.

| Flag | Default | Description |
|------|---------|-------------|
| `--token-a` | required | First token |
| `--token-b` | required | Second token |
| `--amount-a` | required | Desired amount of token_a |
| `--amount-b` | required | Desired amount of token_b |
| `--stable` | false | Add to stable pool |
| `--slippage` | `0.5` | Slippage tolerance % |
| `--deadline-minutes` | `20` | Tx deadline |
| `--confirm` | false | Broadcast on-chain |
| `--dry-run` | false | Build calldata only |

---

### `remove-liquidity` — Withdraw from pool

```bash
# Remove 50% of your position:
aerodrome-amm remove-liquidity --token-a WETH --token-b USDC --percent 50 --confirm

# Remove exact LP amount:
aerodrome-amm remove-liquidity --token-a WETH --token-b USDC --liquidity 0.001 --confirm

# Remove 100% from stable pool:
aerodrome-amm remove-liquidity --token-a USDC --token-b USDT --percent 100 --stable --confirm
```

Approves LP tokens (pool contract) to the Router, then calls `removeLiquidity`. Run `positions` first to see your LP balance.

| Flag | Default | Description |
|------|---------|-------------|
| `--token-a` | required | First token |
| `--token-b` | required | Second token |
| `--liquidity` | — | Exact LP amount to burn |
| `--percent` | — | Percentage of LP balance (1–100) |
| `--stable` | false | Remove from stable pool |
| `--slippage` | `0.5` | Slippage tolerance % |
| `--confirm` | false | Broadcast on-chain |
| `--dry-run` | false | Build calldata only |

One of `--liquidity` or `--percent` is required.

---

### `claim-fees` — Collect trading fees

```bash
# Preview:
aerodrome-amm claim-fees --token-a WETH --token-b USDC

# Execute:
aerodrome-amm claim-fees --token-a WETH --token-b USDC --confirm
aerodrome-amm claim-fees --token-a USDC --token-b USDT --stable --confirm
```

Calls `claimFees()` on the pool. Accrued trading fees (proportional to your pool share and trading volume) are sent directly to your wallet. Fee amounts are determined on-chain at execution time.

| Flag | Default | Description |
|------|---------|-------------|
| `--token-a` | required | First token |
| `--token-b` | required | Second token |
| `--stable` | false | Claim from stable pool |
| `--confirm` | false | Broadcast on-chain |
| `--dry-run` | false | Build calldata only |

---

## Pool Types: Volatile vs Stable

| | Volatile | Stable |
|-|---------|--------|
| AMM formula | xy=k | x^3y + xy^3 = k |
| Best for | Uncorrelated (WETH/USDC, WETH/AERO) | Pegged (USDC/USDT, EURC/USDC) |
| Price impact | Higher for large trades | Lower for correlated assets |
| `--stable` flag | not needed | required |

The `swap` and `quote` commands automatically try both and pick the better output.

---

## Supported Tokens

| Symbol | Address (Base) |
|--------|---------------|
| WETH | `0x4200000000000000000000000000000000000006` |
| USDC | `0x833589fcd6edb6e08f4c7c32d4f71b54bda02913` |
| AERO | `0x940181a94a35a4569e4529a3cdfb74e38fd98631` |
| USDT | `0xfde4c96c8593536e31f229ea8f37b2ada2699bb2` |
| DAI  | `0x50c5725949a6f0c72e6c4a641f24049a917db0cb` |
| cbETH | `0x2ae3f1ec7f1f5012cfeab0185bfc7aa3cf0dec22` |
| cbBTC | `0xcbb7c0000ab88b473b1f5afd9ef808440eed33bf` |
| EURC | `0x60a3e35cc302bfa44cb288bc5a4f316fdb1adb42` |

Any ERC-20 with an Aerodrome AMM pool can be used by passing its address directly.

---

## Changelog

### v0.1.0 (2026-04-28)
- Initial release: 8 commands covering full AMM lifecycle on Base
- Volatile and stable pool support with auto-selection on best output
- `quoteAddLiquidity` preview for accurate add-liquidity estimates
- On-chain allowance checks before approval (idempotent)
- `claimFees()` for LP fee collection
