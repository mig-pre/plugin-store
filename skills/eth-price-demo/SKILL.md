---
name: eth-price-demo
description: "Query ETH price via OnchainOS — a simple demo plugin for CI testing"
license: MIT
metadata:
  author: yz06276
  version: "0.1.0"
---

# ETH Price Demo

A minimal demo plugin that queries the current ETH price. Uses OnchainOS CLI when available, falls back to the OKX public API.

## Commands

### get-price — Get Current ETH Price

```bash
eth-price-demo get-price [--chain <CHAIN_ID>]
```

**Parameters:**
| Parameter | Default | Description |
|-----------|---------|-------------|
| `--chain` | `1` | Chain ID (1 = Ethereum mainnet) |

**Example:**
```bash
eth-price-demo get-price
eth-price-demo get-price --chain 1
```

**Output (JSON):**
```json
{
  "ok": true,
  "token": "ETH",
  "chain_id": "1",
  "price_usd": "1823.45",
  "open_24h": "1810.20",
  "high_24h": "1850.00",
  "low_24h": "1795.30",
  "volume_24h": "5234567.89",
  "source": "okx-public-api"
}
```

## Data Sources

1. **OnchainOS CLI** (preferred): `onchainos dex token price-info --chain 1 --token 0xEeee...`
2. **OKX Public API** (fallback): `GET https://www.okx.com/api/v5/market/ticker?instId=ETH-USDT`

## Safety

- Read-only plugin — no transactions, no wallet access
- No API keys required (uses public endpoints)
- No sensitive data stored or transmitted
