# Hyperliquid CLI — Command Reference

Complete reference for all 13 commands, return fields, authentication, key concepts, and edge cases.

---

## Authentication

| Command Group | Auth Required | Method |
|---------------|---------------|--------|
| markets, spot-markets, price, orderbook, funding | No | Public API |
| buy, sell, cancel, positions, balances, orders, deposit, withdraw | Yes | onchainos wallet login + local key |

Trading commands use a local Hyperliquid trading key auto-generated at `~/.config/dapp-hyperliquid/key.hex`. The user only needs to be logged in to their onchainos wallet:

```bash
# Login once
onchainos wallet login
# All signing is handled automatically from here
```

**Signing flow for exchange actions** (handled internally):
1. Msgpack-encode the action + nonce + vault flag → keccak256 → `connectionId`
2. Build EIP-712 typed data: `Agent { source="a"(mainnet)/"b"(testnet), connectionId }`
3. Sign digest with local k256 key at `~/.config/dapp-hyperliquid/key.hex` → `{ r, s, v }`

**Signing flow for deposit** (handled internally):
1. onchainos sends USDC from AA wallet to the local Hyperliquid trading key address
2. Local key signs EIP-2612 USDC permit (domain: "USD Coin", chainId=42161)
3. onchainos calls Hyperliquid Bridge2 with `batchedDepositWithPermit` (user = local key address)

**Signing flow for withdraw** (handled internally):
1. Local key signs EIP-712 `HyperliquidTransaction:Withdraw` (domain: "HyperliquidSignTransaction", chainId=421614)
2. Signed payload posted directly to Hyperliquid `/exchange`

---

## 1. dapp-hyperliquid markets

List all perpetual futures markets with current mid prices and leverage limits.

```bash
dapp-hyperliquid markets
```

No parameters. No auth required.

**Return fields (per market):**

| Field | Description |
|-------|-------------|
| `symbol` | Asset symbol (e.g. BTC, ETH, SOL) |
| `mid_price` | Current mid price (string) |
| `szDecimals` | Size decimal precision — orders must respect this |
| `maxLeverage` | Maximum allowed leverage for this asset |

---

## 2. dapp-hyperliquid spot-markets

List all spot trading markets.

```bash
dapp-hyperliquid spot-markets
```

No parameters. No auth required.

**Return fields (per market):**

| Field | Description |
|-------|-------------|
| `name` | Market name (e.g. PURR/USDC) |
| `base` | Base token symbol |
| `quote` | Quote token symbol |
| `index` | Universe index (spot asset index = 10000 + index) |

---

## 3. dapp-hyperliquid price

Get the current mid price for a symbol.

```bash
dapp-hyperliquid price <symbol>
```

| Param | Required | Description |
|-------|----------|-------------|
| `<symbol>` | Yes | Asset symbol (e.g. BTC, ETH, SOL, PURR) |

No auth required.

**Return fields:**

| Field | Description |
|-------|-------------|
| `symbol` | Asset symbol |
| `mid_price` | Current mid price (best bid + best ask) / 2 |

**Error:** `symbol 'X' not found in allMids` — use `markets` or `spot-markets` to see valid symbols.

---

## 4. dapp-hyperliquid orderbook

Get the L2 order book snapshot for a symbol.

```bash
dapp-hyperliquid orderbook <symbol>
```

| Param | Required | Description |
|-------|----------|-------------|
| `<symbol>` | Yes | Asset symbol |

No auth required.

**Return fields:**

| Field | Description |
|-------|-------------|
| `coin` | Asset symbol |
| `levels[0]` | Bid levels (array of `{px, sz, n}`) — sorted best-to-worst |
| `levels[1]` | Ask levels (array of `{px, sz, n}`) — sorted best-to-worst |
| `time` | Timestamp (milliseconds) |

Per level: `px` = price, `sz` = total size, `n` = number of orders at that level.

---

## 5. dapp-hyperliquid funding

Get current and 24h historical funding rates for a symbol.

```bash
dapp-hyperliquid funding <symbol>
```

| Param | Required | Description |
|-------|----------|-------------|
| `<symbol>` | Yes | Asset symbol (perp only) |

No auth required.

**Return fields:**

| Field | Description |
|-------|-------------|
| `symbol` | Asset symbol |
| `current_funding` | Current funding rate from meta (may be null) |
| `history_24h` | Array of `{coin, fundingRate, premium, time}` for the past 24h |

**Interpreting funding rate:**
- Positive → longs pay shorts (bearish pressure)
- Negative → shorts pay longs (bullish pressure)
- Rate is per hour; annualized = rate × 8760

---

## 6. dapp-hyperliquid buy

Place a limit buy order (long perp or spot buy). Optionally set leverage first.

```bash
dapp-hyperliquid buy --symbol <symbol> --size <size> --price <price> [--leverage <leverage>]
```

| Param | Required | Default | Description |
|-------|----------|---------|-------------|
| `--symbol` | Yes | — | Asset symbol (e.g. BTC, ETH) |
| `--size` | Yes | — | Order size in base asset units (respect `szDecimals`) |
| `--price` | Yes | — | Limit price in USD |
| `--leverage` | No | Current account setting | Leverage multiplier (1–50, varies by asset) |

Requires onchainos wallet login.

**Return fields:**

| Field | Description |
|-------|-------------|
| `action` | "buy" |
| `symbol` | Asset symbol |
| `size` | Order size as submitted |
| `price` | Limit price as submitted |
| `leverage` | Leverage used (null if not set) |
| `result` | Raw Hyperliquid exchange response |

**Notes:**
- Price and size are normalized (trailing zeros stripped) before signing — required by Hyperliquid
- If `--leverage` is set, a separate `updateLeverage` action is submitted first (cross margin)
- Order type: GTC limit (`{"limit": {"tif": "Gtc"}}`)
- If account does not exist, the CLI will prompt: `dapp-hyperliquid deposit --amount <USDC>`

---

## 7. dapp-hyperliquid sell

Place a limit sell order (short perp or spot sell).

```bash
dapp-hyperliquid sell --symbol <symbol> --size <size> --price <price>
```

| Param | Required | Description |
|-------|----------|-------------|
| `--symbol` | Yes | Asset symbol |
| `--size` | Yes | Order size in base asset units |
| `--price` | Yes | Limit price in USD |

Requires onchainos wallet login.

**Return fields:**

| Field | Description |
|-------|-------------|
| `action` | "sell" |
| `symbol` | Asset symbol |
| `size` | Order size as submitted |
| `price` | Limit price as submitted |
| `result` | Raw Hyperliquid exchange response |

---

## 8. dapp-hyperliquid cancel

Cancel an open order by symbol and order ID.

```bash
dapp-hyperliquid cancel --symbol <symbol> --order-id <order-id>
```

| Param | Required | Description |
|-------|----------|-------------|
| `--symbol` | Yes | Asset symbol the order was placed on |
| `--order-id` | Yes | Order ID (from `orders` command or buy/sell response) |

Requires onchainos wallet login.

**Return fields:**

| Field | Description |
|-------|-------------|
| `action` | "cancel" |
| `symbol` | Asset symbol |
| `order_id` | Order ID that was cancelled |
| `result` | Raw Hyperliquid exchange response |

---

## 9. dapp-hyperliquid positions

View all open perpetual positions for the wallet.

```bash
dapp-hyperliquid positions
```

No parameters. Requires onchainos wallet login (address resolved automatically).

**Return fields:**

| Field | Description |
|-------|-------------|
| `positions` | Array of open perp positions (`assetPositions`) |
| `margin_summary` | Account-level margin summary (total value, margin used, etc.) |
| `cross_margin_summary` | Cross-margin specific summary |

Per position fields (from Hyperliquid): `position.coin`, `position.szi` (size, negative = short), `position.entryPx`, `position.unrealizedPnl`, `position.leverage`, `position.liquidationPx`, `position.marginUsed`, `position.returnOnEquity`.

---

## 10. dapp-hyperliquid balances

View USDC perpetual margin balance and spot token balances.

```bash
dapp-hyperliquid balances
```

No parameters. Requires onchainos wallet login.

**Return fields:**

| Field | Description |
|-------|-------------|
| `perps_margin` | Margin summary for perpetuals account (accountValue, marginUsed, withdrawable) |
| `spot_balances` | Array of spot token balances `[{coin, hold, total, entryNtl}]` |

---

## 11. dapp-hyperliquid orders

List open orders, optionally filtered by symbol.

```bash
dapp-hyperliquid orders [--symbol <symbol>]
```

| Param | Required | Description |
|-------|----------|-------------|
| `--symbol` | No | Filter to a specific asset symbol |

Requires onchainos wallet login.

**Return fields:**

| Field | Description |
|-------|-------------|
| `orders` | Array of open orders |

Per order fields (from Hyperliquid): `coin`, `side` ("B"=buy/"A"=sell), `limitPx`, `sz`, `oid` (order ID), `timestamp`, `origSz`.

---

## Decimal Normalization

Hyperliquid normalizes price and size strings before hashing — the CLI does this automatically:

```
"0.170" → "0.17"
"58.00" → "58"
"100"   → "100"
```

Always match `szDecimals` from `markets` when specifying `--size`. Orders with extra decimal places are rejected.

---

## Common Workflows

### Research then Trade

```bash
dapp-hyperliquid markets                          # find symbol + maxLeverage
dapp-hyperliquid funding BTC                      # check funding (positive = bearish)
dapp-hyperliquid price BTC                        # get current mid price
dapp-hyperliquid orderbook BTC                    # check spread and depth
dapp-hyperliquid buy --symbol BTC --size 0.001 --price 70000 --leverage 10
dapp-hyperliquid positions                        # verify position opened
```

### Position Management

```bash
dapp-hyperliquid positions                        # see all open positions
dapp-hyperliquid orders                           # list pending orders
dapp-hyperliquid cancel --symbol BTC --order-id 123456
dapp-hyperliquid sell --symbol BTC --size 0.001 --price 71000
```

### Spot Trading

```bash
dapp-hyperliquid spot-markets                     # browse spot pairs
dapp-hyperliquid price PURR                       # check spot price
dapp-hyperliquid buy --symbol PURR --size 100 --price 0.09
```

### Withdraw Funds

```bash
dapp-hyperliquid balances                         # check withdrawable USDC
dapp-hyperliquid withdraw --amount 20             # withdraw $20 to your onchainos AA wallet
# Wait ~10–30 min for USDC to appear on Arbitrum
```

---

## 12. dapp-hyperliquid deposit

Deposit USDC from Arbitrum One to open or fund your Hyperliquid account. Uses EIP-2612 gasless permit — no separate approve transaction needed. Signs a permit off-chain, then calls `batchedDepositWithPermit` on the Hyperliquid bridge in a single transaction.

```bash
dapp-hyperliquid deposit --amount <amount>
```

| Param | Required | Description |
|-------|----------|-------------|
| `--amount` | Yes | USDC amount (e.g. `10` for $10.00, `50.5` for $50.50) |

Requires onchainos wallet login. Uses USDC on Arbitrum One and the Hyperliquid bridge contract.

**Return fields:**

| Field | Description |
|-------|-------------|
| `action` | "deposit" |
| `amount_usdc` | Amount deposited |
| `deposit_tx` | Arbitrum UserOp hash for the bridge deposit |
| `note` | Account activation time (~1 minute) |

**Minimum deposit:** $5 USDC. Amounts below $5 are permanently lost.

**Timing:** ~1 minute after the Arbitrum transaction confirms before the balance appears on Hyperliquid.

**Contract addresses (Arbitrum One):**
- USDC: `0xaf88d065e77c8cC2239327C5EDb3A432268e5831`
- Hyperliquid Bridge2: `0x2df1c51e09aecf9cacb7bc98cb1742757f163df7`

---

## 13. dapp-hyperliquid withdraw

Withdraw USDC from your Hyperliquid account back to Arbitrum One. Signed locally using the Hyperliquid trading key — no bridge approval transaction required.

```bash
dapp-hyperliquid withdraw --amount <amount> [--destination <address>]
```

| Param | Required | Description |
|-------|----------|-------------|
| `--amount` | Yes | USDC amount to withdraw (e.g. `5` for $5.00, `10.5` for $10.50) |
| `--destination` | No | Arbitrum destination address (default: your onchainos AA wallet address) |

Requires onchainos wallet login. Uses the local Hyperliquid trading key for signing.

**Return fields:**

| Field | Description |
|-------|-------------|
| `action` | "withdraw" |
| `amount` | Amount withdrawn |
| `destination` | Destination Arbitrum address |
| `result` | Raw Hyperliquid exchange response |

**Timing:** ~10–30 minutes for USDC to appear on Arbitrum after Hyperliquid processes the withdrawal.

**Minimum withdrawal:** $2 USDC (Hyperliquid enforces this).

**Notes:**
- The destination defaults to your onchainos AA wallet — pass `--destination` to override
- Withdrawal is processed by Hyperliquid L1; bridging to Arbitrum happens asynchronously
- Use `balances` first to check your `withdrawable` amount

---

## Edge Cases & Errors

| Error | Cause | Fix |
|-------|-------|-----|
| `onchainos wallet not available` | Not logged in | `onchainos wallet login` |
| `Hyperliquid account not found` | No account on HL yet | Run `deposit --amount <USDC>` first |
| `symbol 'X' not found` | Invalid symbol | Run `markets` or `spot-markets` first |
| `Rate limited` | Too many requests | Retry with backoff |
| Order rejected | Wrong `szDecimals` | Check precision via `markets` |
| Order rejected | Insufficient margin | Check `balances` first, or `deposit` more USDC |
| Leverage > maxLeverage | Exceeds asset limit | Check `maxLeverage` from `markets` |
| Self-trade prevention | Would cross your own order | Cancel existing order first |
| `contract-call failed` on deposit | Insufficient USDC on Arbitrum | Bridge USDC to Arbitrum first |
| Withdraw amount too low | Below $2 minimum | Withdraw at least $2 USDC |
| Withdraw not arriving | Hyperliquid processing delay | Wait up to 30 min; check Hyperliquid UI |

---

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `HYPERLIQUID_URL` | No | Override API base URL (default: `https://api.hyperliquid.xyz`). Set to testnet URL for testing. |
