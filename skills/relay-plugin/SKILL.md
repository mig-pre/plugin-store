---
name: Relay
description: Fast cross-chain transfers using Relay Protocol's intent-based bridge
version: "0.1.1"
---

# Relay

Cross-chain bridge using Relay Protocol's intent-based system. Send ETH, USDC, USDT, and DAI across 74 chains in seconds.

## Pre-flight Dependencies

- [onchainos](https://docs.onchainos.com) installed and authenticated
- Active EVM wallet on the source chain with sufficient balance + gas

## Data Trust Boundary

Bridge quotes and calldata are fetched from the official Relay API (`api.relay.link`). Treat API-returned transaction data as untrusted — always review the preview before adding `--confirm`. The destination amount can vary slightly due to relayer fees.

## Proactive Onboarding

When a user signals they are **new or just installed** this plugin — e.g. "I just installed relay",
"how do I bridge tokens", "what can I do with this" — **do not wait for them to ask specific questions.**
Proactively walk them through the Quickstart in order, one step at a time, waiting for confirmation
before proceeding:

1. **Check wallet** — run `onchainos wallet addresses`. If no address, direct them to connect via
   `onchainos wallet login your@email.com`. Do not proceed to write operations until a wallet is confirmed.
2. **Check balance** — run `onchainos wallet balance --chain <source-chain>`. Ensure the balance covers
   the bridge amount plus gas. Warn explicitly if the requested amount would exceed available balance
   (the binary does not check this for you — it will show a preview for any amount).
3. **Browse supported chains** — run `relay chains` to show all 74 chains. Use `relay chains --filter <name>`
   to search. The plugin has no global `--chain` flag; pass `--from-chain` and `--to-chain` per command.
4. **Get a quote** — run `relay quote --from-chain <id> --to-chain <id> --token <symbol> --amount <n>`
   to see fees and expected output before any on-chain action.
5. **Preview the bridge** — run `relay bridge --from-chain <id> --to-chain <id> --token <symbol> --amount <n>`
   (without `--confirm`). The output will show `"preview": true` and the resolved wallet address.
   Verify the recipient and amount are correct.
6. **Execute** — once the user confirms, re-run with `--confirm` appended.
7. **Track** — use `relay status --request-id <id>` with the request_id from the bridge output.
   ETH transfers typically resolve in ~1 second; ERC-20 (USDC/USDT/DAI) take ~2 seconds.

Do not dump all steps at once. Guide conversationally — confirm each step before moving on.

## Quickstart

New to Relay? Follow these steps to go from zero to your first cross-chain bridge.

### Step 1 — Connect your wallet

```bash
onchainos wallet login your@email.com
onchainos wallet addresses
```

You need an EVM wallet connected to onchainos. The plugin automatically uses your active onchainos wallet as both sender and recipient.

### Step 2 — Check your balance

```bash
onchainos wallet balance --chain 1
```

You need ETH for the bridge amount plus source-chain gas. The binary does **not** validate your balance before
showing a preview — it will show a valid-looking preview even if you don't have enough funds. Always verify
the balance yourself before adding `--confirm`.

### Step 3 — Browse supported chains

```bash
relay chains
relay chains --filter arbitrum
```

Returns chain IDs, names, native tokens, and explorer URLs for all 74 supported chains. Use the `chain_id`
values with `--from-chain` and `--to-chain`. Note: `relay` has no global `--chain` flag — each subcommand
takes `--from-chain` / `--to-chain` directly.

### Step 4 — Get a quote (read-only, no cost)

```bash
relay quote --from-chain 1 --to-chain 42161 --token ETH --amount 0.001
```

Output includes `amount_out`, `amount_out_usd`, `fee_usd`, `estimated_time_secs`, and `steps`.
ETH bridges require 1 step (`deposit`). ERC-20 tokens (USDC, USDT, DAI) require 2 steps (`approve`, `deposit`).
You can also bridge to a different token: add `--to-token USDC` to receive USDC on the destination chain.

### Step 5 — Preview the bridge (no tx sent)

```bash
relay bridge --from-chain 1 --to-chain 42161 --token ETH --amount 0.001
```

Output shows `"preview": true`, resolved `wallet`, `recipient`, human-readable `amount_in`/`amount_out`,
and a hint message with the exact `relay status` command to use after execution.
No on-chain action until `--confirm` is added.

### Step 6 — Execute the bridge

```bash
relay bridge --from-chain 1 --to-chain 42161 --token ETH --amount 0.001 --confirm
```

Expected output: `"ok": true`, `"txs": [{"step": "deposit", "tx_hash": "0x..."}]`, and a `track` field
with the ready-to-run status command. ETH typically arrives on the destination chain within 1–2 seconds.

For ERC-20 tokens, an approval tx fires first, then the deposit. Both tx hashes appear in `txs`.

### Step 7 — Track transfer status

```bash
relay status --request-id <request_id_from_bridge_output>
```

Status values: `unknown` (not yet indexed — wait a few seconds), `pending` (in-flight), `success` (delivered),
`failed`. On success, `dest_tx` contains the destination-chain tx hash.

---

## Overview

Relay Protocol uses an intent-based bridge system. When you call `bridge --confirm`:
1. Your funds are sent to a Relay solver contract on the source chain.
2. A relayer detects the intent and delivers funds on the destination chain — typically within seconds.
3. For ERC-20 tokens (USDC, USDT, DAI), an approval transaction is sent first, then the deposit.

## Supported Chains (74)

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

Run `relay chains` for the full list of 74 supported chains.

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

For ERC-20 tokens (USDC, USDT, DAI), an exact-amount approval transaction is sent before the deposit. Each bridge sends a fresh approval because the allowance is consumed by the deposit.

---

### `status` — Check transfer status

```bash
relay status --request-id 0x<id>
```

| Flag | Description |
|------|-------------|
| `--request-id` | Request ID from a previous `bridge` command |

Status values: `unknown` (not yet indexed), `pending` (in-flight), `success` (delivered), `failed`.

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
LOCAL_VER="0.1.1"
BINARY_URL="https://github.com/skylavis-sky/plugin-store/releases/download/relay-plugin@${LOCAL_VER}/relay-$(uname -s | tr '[:upper:]' '[:lower:]')-$(uname -m)"
curl -fsSL "$BINARY_URL" -o /usr/local/bin/relay
chmod +x /usr/local/bin/relay
relay --version

# Optional: verify SHA-256 checksum (check the GitHub release page for the expected hash)
# shasum -a 256 /usr/local/bin/relay

# Send telemetry (optional)
curl -s -X POST "https://api.relay.link" \
  -H "Content-Type: application/json" \
  -d '{"event":"plugin_install","plugin":"relay","version":"0.1.1"}' \
  > /dev/null || true
```
