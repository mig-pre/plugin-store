---
name: hl-funding-flow-hunter
description: "Hyperliquid funding and flow strategy scanner with guarded execution"
version: "1.0.0"
author: "KB"
tags:
  - hyperliquid
  - perpetuals
  - funding
  - trading-strategy
  - risk-management
---


## Pre-flight Dependencies (auto-injected by Plugin Store CI)

> Run once per session before first use. These checks ensure required tools are installed.

### Version check (auto-injected)

> **AI Agent Instruction**: Run this version check before executing any commands from this skill. If an update is applied, re-read this SKILL.md before proceeding — the instructions may have changed.

```bash
# Check for skill updates (1-hour cache)
UPDATE_CACHE="$HOME/.plugin-store/update-cache/hl-funding-flow-hunter"
CACHE_MAX=3600
LOCAL_VER="1.0.0"
DO_CHECK=true

if [ -f "$UPDATE_CACHE" ]; then
  CACHE_MOD=$(stat -f %m "$UPDATE_CACHE" 2>/dev/null || stat -c %Y "$UPDATE_CACHE" 2>/dev/null || echo 0)
  NOW=$(date +%s)
  AGE=$(( NOW - CACHE_MOD ))
  [ "$AGE" -lt "$CACHE_MAX" ] && DO_CHECK=false
fi

if [ "$DO_CHECK" = true ]; then
  REMOTE_VER=$(curl -sf --max-time 3 "https://raw.githubusercontent.com/okx/plugin-store/main/skills/hl-funding-flow-hunter/plugin.yaml" | grep '^version' | head -1 | tr -d '"' | awk '{print $2}')
  if [ -n "$REMOTE_VER" ]; then
    mkdir -p "$HOME/.plugin-store/update-cache"
    echo "$REMOTE_VER" > "$UPDATE_CACHE"
  fi
fi

REMOTE_VER=$(cat "$UPDATE_CACHE" 2>/dev/null || echo "$LOCAL_VER")
if [ "$REMOTE_VER" != "$LOCAL_VER" ]; then
  echo "Update available: hl-funding-flow-hunter v$LOCAL_VER -> v$REMOTE_VER. Updating..."
  npx skills add okx/plugin-store --skill hl-funding-flow-hunter --yes --global 2>/dev/null || true
  echo "Updated hl-funding-flow-hunter to v$REMOTE_VER. Please re-read this SKILL.md."
fi
```

### Report install (auto-injected, runs once)

```bash
REPORT_FLAG="$HOME/.plugin-store/reported/hl-funding-flow-hunter"
if [ ! -f "$REPORT_FLAG" ]; then
  mkdir -p "$HOME/.plugin-store/reported"
  # Device fingerprint → SHA256 → 32-char device ID
  DEV_RAW="$(hostname):$(uname -s):$(uname -m):$HOME"
  DEV_ID=$(echo -n "$DEV_RAW" | shasum -a 256 | head -c 32)
  # HMAC signature (obfuscated key, same as CLI binary)
  _K=$(echo 'OE9nNWFRUFdfSVJkektrMExOV2RNeTIzV2JibXo3ZWNTbExJUDFIWnVoZw==' | base64 -d 2>/dev/null || echo 'OE9nNWFRUFdfSVJkektrMExOV2RNeTIzV2JibXo3ZWNTbExJUDFIWnVoZw==' | openssl base64 -d)
  HMAC_SIG=$(echo -n "${_K}${DEV_ID}" | shasum -a 256 | head -c 8)
  DIV_ID="${DEV_ID}${HMAC_SIG}"
  unset _K
  # Report to Vercel stats
  curl -s -X POST "https://plugin-store-dun.vercel.app/install" \
    -H "Content-Type: application/json" \
    -d '{"name":"hl-funding-flow-hunter","version":"1.0.0"}' >/dev/null 2>&1 || true
  # Report to OKX API (with HMAC-signed device token)
  curl -s -X POST "https://www.okx.com/priapi/v1/wallet/plugins/download/report" \
    -H "Content-Type: application/json" \
    -d '{"pluginName":"hl-funding-flow-hunter","divId":"'"$DIV_ID"'"}' >/dev/null 2>&1 || true
  touch "$REPORT_FLAG"
fi
```

---


# HL Funding Flow Hunter

## Overview

This skill scans Hyperliquid perpetual markets for funding-rate and flow opportunities, then helps the user prepare guarded low-leverage execution through `hyperliquid-plugin`. The bundled scanner is read-only: it queries public Hyperliquid market data and never places orders.

Live trading must always use `hyperliquid order` or `hyperliquid close` from `hyperliquid-plugin`, and every write command must include `--strategy-id hl-funding-flow-hunter`.

## Pre-flight Checks

Before using this skill:

1. Install the dependency:

```bash
npx skills add okx/plugin-store --skill hyperliquid-plugin --yes
```

2. Ensure the Hyperliquid account is registered:

```bash
hyperliquid register
```

3. Check setup and balances:

```bash
hyperliquid quickstart
```

4. Explain that perpetual futures can lose more than the expected funding benefit, especially when leverage is used.

## Commands

### Scan Funding and Flow

```bash
python3 ./scripts/scan_funding.py --top 5
```

Run this command from the `hl-funding-flow-hunter` skill directory. If the agent is operating from a plugin-store checkout, use `python3 skills/hl-funding-flow-hunter/scripts/scan_funding.py --top 5` instead.

**When to use**: When the user asks for funding candidates, flow opportunities, or a low-leverage Hyperliquid perp setup.
**Output**: JSON with ranked candidates, suggested side, funding, volume, open interest base, open interest notional, score, and risk flags.
**Safety**: This command is read-only and does not prepare or submit orders.

### Get Current Price

```bash
hyperliquid prices --coin ETH
```

**When to use**: Before sizing an order, setting stop loss, or setting take profit.
**Output**: Current Hyperliquid price information for the selected coin.

### Dry-run Order

```bash
hyperliquid order \
  --coin <COIN> \
  --side sell \
  --size <SIZE> \
  --type market \
  --leverage 2 \
  --isolated \
  --sl-px <STOP_LOSS_PRICE> \
  --tp-px <TAKE_PROFIT_PRICE> \
  --dry-run \
  --strategy-id hl-funding-flow-hunter
```

**When to use**: Before any live order, after the user chooses a candidate, direction, size, leverage, stop loss, and take profit.
**Output**: A preview of the order without placing a real trade.
**Required**: Use `--strategy-id hl-funding-flow-hunter` on every dry-run and live write command.

### Confirmed Order

```bash
hyperliquid order \
  --coin <COIN> \
  --side sell \
  --size <SIZE> \
  --type market \
  --leverage 2 \
  --isolated \
  --sl-px <STOP_LOSS_PRICE> \
  --tp-px <TAKE_PROFIT_PRICE> \
  --confirm \
  --strategy-id hl-funding-flow-hunter
```

**When to use**: Only after the user explicitly confirms the coin, side, size, leverage, stop loss, take profit, and risk budget.
**Output**: A Hyperliquid order result from the dependency plugin.

### Dry-run Close

```bash
hyperliquid close \
  --coin <COIN> \
  --dry-run \
  --strategy-id hl-funding-flow-hunter
```

**When to use**: Before closing a live position.
**Output**: A preview of the close action without placing a real trade.

### Confirmed Close

```bash
hyperliquid close \
  --coin <COIN> \
  --confirm \
  --strategy-id hl-funding-flow-hunter
```

**When to use**: Only after the user explicitly confirms the close.
**Output**: A Hyperliquid close result from the dependency plugin.

## Strategy Workflow

Use this workflow when the user asks for a funding or flow strategy:

1. Run the scanner:

```bash
python3 ./scripts/scan_funding.py --top 5
```

2. Present the top candidates with:

- coin
- suggested side
- funding rate
- estimated annualized funding
- 24h volume
- open interest base and open interest notional
- score
- risk flags

3. Ask the user to choose a candidate or request alternatives.
4. Get current price with `hyperliquid prices --coin <COIN>`.
5. Convert the user's intended USDC notional into coin size. Hyperliquid `--size` is the coin quantity, not USDC amount.
6. Propose conservative risk parameters:

- isolated margin
- 1-2x leverage by default
- maximum 3x leverage
- stop loss before live execution
- take profit or funding-normalization exit rule
- maximum holding time

7. Run a dry-run order first.
8. Ask for explicit confirmation.
9. If confirmed, execute the live order with `--confirm --strategy-id hl-funding-flow-hunter`.
10. Track the planned exit conditions and use `hyperliquid close` only after confirmation.

## Candidate Presentation Template

```text
Top candidate: <COIN>
Suggested side: SELL
Reason: positive funding, sufficient volume, acceptable open interest notional
Leverage: 2x isolated
Estimated notional: <USDC>
Estimated margin: <USDC>
Coin size: <SIZE>
Stop loss: <SL>
Take profit: <TP>
Exit conditions: funding normalizes, stop loss hits, take profit hits, or max holding time is reached
Strategy ID: hl-funding-flow-hunter

Reply confirm to dry-run, alternatives to see more candidates, or skip.
```

## Guardrails

- The scanner is read-only and must never call Hyperliquid exchange order APIs directly.
- Live orders must go through `hyperliquid-plugin`.
- Every order and close command must include `--strategy-id hl-funding-flow-hunter`.
- Default to isolated margin.
- Default to 1-2x leverage; do not exceed 3x.
- Suggested single-coin margin cap: 20% of account equity.
- Suggested session risk budget: 5% of account equity.
- Exclude or warn on low 24h volume, low open interest notional, thin liquidity, or fast funding reversals.
- Always run a dry-run before live execution.
- Never support unlimited autonomous trading.
- Never ask for private keys, seed phrases, API secrets, or email OTP codes.

## Examples

### Scan and Prepare a Candidate

User: "Find the best Hyperliquid funding setup under 500 USDC margin."

Agent:

1. Runs `python3 ./scripts/scan_funding.py --top 5` from the skill directory.
2. Shows the top 3 candidates and risk flags.
3. Asks the user to choose one.
4. Gets current price and calculates size.
5. Runs a dry-run order only after the user accepts the proposed parameters.
6. Runs the live order only after explicit confirmation.

### Live Order After Confirmation

```bash
hyperliquid order \
  --coin ETH \
  --side sell \
  --size <SIZE> \
  --type market \
  --leverage 2 \
  --isolated \
  --sl-px <STOP_LOSS_PRICE> \
  --tp-px <TAKE_PROFIT_PRICE> \
  --confirm \
  --strategy-id hl-funding-flow-hunter
```

## Error Handling

| Error | Cause | Resolution |
|-------|-------|------------|
| `hyperliquid_api_unavailable` | Public Hyperliquid info API failed or timed out | Wait and retry, or lower `--top`; do not trade without fresh data |
| `unexpected_hyperliquid_response` | API response shape changed | Report the raw error and avoid live execution until the scanner is updated |
| No candidates | Filters removed all markets | Relax read-only filters or ask the user to try later |
| Invalid size | Hyperliquid size is coin quantity, not USDC notional | Recalculate size from current price and coin decimals |
| Missing confirmation | Live write has not been approved | Ask the user to confirm coin, side, size, leverage, stop loss, take profit, and risk budget |
| Insufficient margin | Account lacks available USDC | Ask the user whether they want deposit/setup guidance from `hyperliquid quickstart` |

## Security Notices

- This is a standard-risk trading strategy skill for perpetual futures.
- Funding can reverse quickly, and price movement can overwhelm expected funding income.
- Leverage increases liquidation risk.
- The scanner only reads public market data from `https://api.hyperliquid.xyz`.
- This skill must never request or store private keys, seed phrases, API secrets, or OTP codes.
- This skill is a guarded trading co-pilot, not an unattended trading bot.
