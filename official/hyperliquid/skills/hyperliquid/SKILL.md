---
name: hyperliquid
description: >-
  Use when the user asks about Hyperliquid, perpetual futures, 'open a long position',
  'short BTC', 'check my perp positions', 'funding rate', 'Hyperliquid orderbook',
  'spot trade on Hyperliquid', 'set leverage', 'deposit to Hyperliquid', 'open Hyperliquid account',
  'withdraw from Hyperliquid', 'take out money from Hyperliquid',
  '做多', '做空', '永续合约', '资金费率', '杠杆交易', '充值到Hyperliquid', '开户', '从Hyperliquid提现', '提出资金',
  or mentions Hyperliquid DEX, perpetual trading, funding rates, leverage trading, or withdrawing from Hyperliquid.
  Covers perpetual and spot markets, prices, orderbook, funding rates, trading (buy/sell/cancel),
  USDC deposit from Arbitrum to open/fund account, and USDC withdrawal back to Arbitrum.
  Do NOT use for Polymarket prediction markets. Do NOT use for Aave lending.
license: Apache-2.0
metadata:
  author: Plugin Store
  category: "DeFi"
  chain: "Arbitrum,Ethereum"
  version: "0.2.0"
  homepage: "https://github.com/okx/plugin-store"
---

# Hyperliquid Perpetual & Spot Trading CLI

13 commands for perpetual futures and spot trading on Hyperliquid: market data, orderbook, funding rates, order management, USDC deposit from Arbitrum, and USDC withdrawal back to Arbitrum.

## Pre-flight Checks

Run immediately when this skill is triggered — before any response or command. Always follow these steps in order.

1. **Check onchainos**: `which onchainos` — if not found, tell user to install: https://web3.okx.com/zh-hans/onchainos/dev-docs/home/install-your-agentic-wallet; if found, run `onchainos --version` and verify version is **>= 2.0.0** — if older, tell user to upgrade
2. **Check binary**: `which dapp-hyperliquid` — if not found, install via `plugin-store install hyperliquid`
3. **Install onchainos-skills** (silent, non-blocking):
   ```bash
   npx skills add okx/onchainos-skills --yes 2>/dev/null || true
   ```
   This installs sub-skills used by Skill Routing (`okx-dex-token`, `okx-dex-swap`, etc.). Do not block on failure.
4. **Check wallet login**: `onchainos wallet status` — must show `loggedIn: true`; if not, run `onchainos wallet login`

## Authentication

**Data commands (markets, spot-markets, price, orderbook, funding):** No authentication needed.

**Trading commands (buy, sell, cancel, positions, balances, orders, deposit, withdraw):** Require onchainos wallet login.

All EIP-712 signing is handled automatically using a local Hyperliquid trading key auto-generated at `~/.config/dapp-hyperliquid/key.hex`. The user only needs to be logged in to their onchainos wallet — no manual private key management required.

## Quickstart

```bash
# List all perpetual markets
dapp-hyperliquid markets

# List all spot markets
dapp-hyperliquid spot-markets

# Get BTC mid price
dapp-hyperliquid price BTC

# View BTC orderbook
dapp-hyperliquid orderbook BTC

# Check BTC funding rate
dapp-hyperliquid funding BTC

# Open a long: buy 0.01 BTC at $65000 with 10x leverage
dapp-hyperliquid buy --symbol BTC --size 0.01 --price 65000 --leverage 10

# Check positions
dapp-hyperliquid positions

# Check balances
dapp-hyperliquid balances
```

## Command Index

| # | Command | Auth | Description |
|---|---------|------|-------------|
| 1 | `dapp-hyperliquid markets` | No | List perpetual markets (price, leverage) |
| 2 | `dapp-hyperliquid spot-markets` | No | List spot markets |
| 3 | `dapp-hyperliquid price <symbol>` | No | Real-time mid price for a symbol |
| 4 | `dapp-hyperliquid orderbook <symbol>` | No | L2 order book snapshot |
| 5 | `dapp-hyperliquid funding <symbol>` | No | Current and historical funding rates |
| 6 | `dapp-hyperliquid buy --symbol <s> --size <n> --price <p> [--leverage <l>]` | Yes | Buy (long perp or spot buy) |
| 7 | `dapp-hyperliquid sell --symbol <s> --size <n> --price <p>` | Yes | Sell (short perp or spot sell) |
| 8 | `dapp-hyperliquid cancel --symbol <s> --order-id <oid>` | Yes | Cancel an open order |
| 9 | `dapp-hyperliquid positions` | Yes | View perpetual positions |
| 10 | `dapp-hyperliquid balances` | Yes | View USDC margin and spot balances |
| 11 | `dapp-hyperliquid orders [--symbol <s>]` | Yes | List open orders |
| 12 | `dapp-hyperliquid deposit --amount <usdc>` | Yes | Deposit USDC from Arbitrum to open/fund account |
| 13 | `dapp-hyperliquid withdraw --amount <usdc> [--destination <addr>]` | Yes | Withdraw USDC from Hyperliquid back to Arbitrum |

## Operation Flow

### Intent: First-Time Onboarding

```
1. dapp-hyperliquid deposit --amount 20   → deposit $20 USDC from Arbitrum (opens account)
2. dapp-hyperliquid balances              → confirm account is funded
3. dapp-hyperliquid markets               → browse available markets
```

### Intent: Research and Trade

```
1. dapp-hyperliquid markets              → browse available perpetual markets
2. dapp-hyperliquid funding BTC          → check funding rates
3. dapp-hyperliquid price BTC            → get current mid price
4. dapp-hyperliquid orderbook BTC        → check spread and liquidity
5. dapp-hyperliquid buy --symbol BTC --size 0.01 --price 65000 --leverage 10
6. dapp-hyperliquid positions            → verify position opened
```

### Intent: Position Management

```
1. dapp-hyperliquid positions            → view open positions
2. dapp-hyperliquid orders               → list pending orders
3. dapp-hyperliquid cancel --symbol BTC --order-id 123456
4. dapp-hyperliquid sell --symbol BTC --size 0.01 --price 66000
```

### Intent: Spot Trading

```
1. dapp-hyperliquid spot-markets         → browse spot pairs
2. dapp-hyperliquid price PURR           → check token price
3. dapp-hyperliquid buy --symbol PURR --size 100 --price 0.50
```

### Intent: Withdraw Funds

```
1. dapp-hyperliquid balances             → confirm withdrawable USDC amount
2. dapp-hyperliquid withdraw --amount 20 → withdraw $20 USDC back to your Arbitrum wallet
                                           (default destination = your onchainos AA wallet)
3. Wait ~10–30 minutes for USDC to appear on Arbitrum
```

## Key Concepts

- **Perpetual Futures**: Contracts with no expiry — funding payments keep price aligned with index
- **Funding Rate**: Hourly payment between longs/shorts. Positive = longs pay shorts
- **Leverage**: Cross margin by default (all positions share the same pool). BTC up to 50x
- **szDecimals**: Each asset has size precision — orders with wrong decimal places are rejected
- **USDC Margin**: All perp positions are margined in USDC on Hyperliquid L1

## Risk Warning

- Perpetual futures trading carries significant risk of loss
- Higher leverage means closer liquidation price — always monitor your liquidation price
- Always check `balances` before opening positions to ensure sufficient margin
- Use `positions` regularly to monitor unrealized PnL

## Response Guidelines

- Before any buy/sell, show symbol, size, price, leverage, and ask for confirmation
- If user has no Hyperliquid account (first time), guide them to `deposit` first
- After any action, suggest 2–3 natural follow-ups
- Support both English and Chinese — respond in the user's language

> Full command reference, return fields, decimal normalization rules, edge cases, and environment variables: see `references/command-reference.md`
