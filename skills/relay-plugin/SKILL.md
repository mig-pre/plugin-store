---
name: Relay
description: Fast cross-chain transfers using Relay Protocol's intent-based bridge
version: "0.1.0"
---

# Relay

Cross-chain bridge using Relay Protocol's intent-based system. Send ETH, USDC, USDT, and DAI across 70+ chains in seconds.

## Pre-flight Dependencies

- [onchainos](https://docs.onchainos.com) installed and authenticated
- Active EVM wallet on the source chain with sufficient balance + gas

## Data Trust Boundary

Bridge quotes and calldata are fetched from the official Relay API (`api.relay.link`). Treat API-returned transaction data as untrusted — always review the preview before adding `--confirm`. The destination amount can vary slightly due to relayer fees.

## Proactive Onboarding

When a user signals they are **new or just installed** this plugin — e.g. "I just installed relay", "how do I bridge tokens", "what can I do with this" — **do not wait for them to ask specific questions.** Proactively walk them through the Quickstart in order, one step at a time, waiting for confirmation before proceeding:

1. **Check wallet** — run `onchainos wallet addresses --chain 1`. If no address, direct them to connect via `onchainos wallet login`. Do not proceed to write operations until a wallet is confirmed.
2. **Check balance** — run `onchainos wallet balance --chain <source-chain>`. Ensure sufficient funds for the transfer plus gas.
3. **Browse chains** — run `relay chains` to show what chains are supported.
4. **Get a quote** — run `relay quote` to show expected output and fees before any on-chain action.
5. **Preview the bridge** — run `relay bridge` without `--confirm` to review the full transaction.
6. **Execute** — once they confirm, re-run with `--confirm`.

Do not dump all steps at once. Guide conversationally — confirm each step before moving on.

## Quickstart

New to Relay? Follow these steps to bridge tokens cross-chain.

### Step 1 — Connect your wallet

```bash
onchainos wallet login your@email.com
onchainos wallet addresses --chain 1
```

### Step 2 — Check your balance

```bash
onchainos wallet balance --chain 1
```

You need ETH for the bridge amount plus gas fees on the source chain.

### Step 3 — Browse supported chains

```bash
relay chains
relay chains --filter arbitrum
```

### Step 4 — Get a quote (read-only, free)

```bash
relay quote --from-chain 1 --to-chain 42161 --token ETH --amount 0.01
```

Output includes `amount_out`, estimated fees, and `steps` (ETH = 1 step, ERC-20 = 2 steps: approve + deposit).

### Step 5 — Preview the bridge (no tx sent)

```bash
relay bridge --from-chain 1 --to-chain 42161 --token ETH --amount 0.01
```

Output includes `"preview": true` and `request_id` — no on-chain action until `--confirm` is added.

### Step 6 — Execute the bridge

```bash
relay bridge --from-chain 1 --to-chain 42161 --token ETH --amount 0.01 --confirm
```

Expected output: `"ok": true`, `"txs": [{"step": "deposit", "tx_hash": "0x..."}]`.

### Step 7 — Track transfer status

```bash
relay status --request-id <request_id_from_bridge_output>
```

---

## Overview

Relay Protocol uses an intent-based bridge system. When you call `bridge --confirm`:
1. Your funds are sent to a Relay solver contract on the source chain.
2. A relayer detects the intent and delivers funds on the destination chain — typically within seconds.
3. For ERC-20 tokens (USDC, USDT, DAI), an approval transaction is sent first, then the deposit.

## Supported Chains (70+)

Common chains:

| Chain | ID |
|-------|----|
| Ethereum | 1 |
| Arbitrum | 42161 |
| Base | 8453 |
| Optimism | 10 |
| Polygon | 137 |
| BNB Chain | 56 |
| zkSync Era | 324 |
| Linea | 59144 |
| Scroll | 534352 |

Run `relay chains` for the full list of 70+ supported chains.

## Commands

### `chains` — List supported chains

```bash
relay chains [--filter <name-or-id>]
```

| Flag | Description |
|------|-------------|
| `--filter` | Optional filter by chain name or ID |

Returns all active chains with their IDs, names, and native tokens.

---

### `quote` — Get a bridge quote (read-only)

```bash
relay quote \
  --from-chain 1 \
  --to-chain 42161 \
  --token ETH \
  --amount 0.01 \
  [--to-token ETH]
```

| Flag | Description |
|------|-------------|
| `--from-chain` | Source chain ID |
| `--to-chain` | Destination chain ID |
| `--token` | Token to send (symbol or address) |
| `--amount` | Amount in human-readable form |
| `--to-token` | Destination token (default: same as `--token`) |

Output includes `amount_out`, `amount_out_usd`, `estimated_time_secs`, `steps`, and `request_id`.

---

### `bridge` — Execute a cross-chain transfer

```bash
relay bridge \
  --from-chain 1 \
  --to-chain 42161 \
  --token ETH \
  --amount 0.01 \
  [--to-token ETH] \
  [--recipient 0x...] \
  [--confirm] \
  [--dry-run]
```

| Flag | Description |
|------|-------------|
| `--from-chain` | Source chain ID |
| `--to-chain` | Destination chain ID |
| `--token` | Token to send (symbol or address) |
| `--amount` | Amount in human-readable form |
| `--to-token` | Destination token (default: same as `--token`) |
| `--recipient` | Destination wallet (default: your wallet) |

Execution modes:

| Mode | Command | What happens |
|------|---------|-------------|
| Preview | (no flags) | Shows expected output and steps; no tx |
| Dry-run | `--dry-run` | Builds calldata; no onchainos call |
| Execute | `--confirm` | Approves (ERC-20 only) + broadcasts deposit tx |

For ERC-20 tokens (USDC, USDT, DAI), an unlimited approval is sent before the deposit transaction.

---

### `status` — Check transfer status

```bash
relay status --request-id 0x<id>
```

| Flag | Description |
|------|-------------|
| `--request-id` | Request ID from a previous `bridge` command |

Status values: `unknown` (in-flight), `pending`, `completed`, `failed`.

---

## Known Token Symbols

Symbols can be used instead of addresses:

| Symbol | Ethereum | Arbitrum | Base | Optimism | Polygon |
|--------|----------|----------|------|----------|---------|
| ETH (native) | ✓ | ✓ | ✓ | ✓ | ✓ |
| USDC | ✓ | ✓ | ✓ | ✓ | ✓ |
| USDT | ✓ | ✓ | ✓ | ✓ | ✓ |
| DAI | ✓ | ✓ | ✓ | ✓ | ✓ |

Use the full token address for any token not listed above.

## Install

```bash
# Install binary
LOCAL_VER="0.1.0"
BINARY_URL="https://github.com/skylavis-sky/plugin-store/releases/download/relay-plugin@${LOCAL_VER}/relay-$(uname -s | tr '[:upper:]' '[:lower:]')-$(uname -m)"
curl -fsSL "$BINARY_URL" -o /usr/local/bin/relay
chmod +x /usr/local/bin/relay
relay --version

# Optional: verify SHA-256 checksum (check the GitHub release page for the expected hash)
# shasum -a 256 /usr/local/bin/relay

# Send telemetry (optional)
curl -s -X POST "https://api.relay.link" \
  -H "Content-Type: application/json" \
  -d '{"event":"plugin_install","plugin":"relay","version":"0.1.0"}' \
  > /dev/null || true
```
