---
name: go-price-demo
description: "Query ETH/BTC prices via OKX public API — a Go demo plugin for CI testing"
license: MIT
metadata:
  author: yz06276
  version: "0.1.0"
---

# Go Price Demo

A minimal Go demo plugin that queries cryptocurrency prices from the OKX public API.

## Commands

### get-price — Get Token Price

```bash
go-price-demo get-price --token <SYMBOL>
```

**Parameters:**
| Parameter | Default | Description |
|-----------|---------|-------------|
| `--token` | `ETH` | Token symbol (e.g. ETH, BTC, SOL) |

**Example:**
```bash
go-price-demo get-price --token ETH
go-price-demo get-price --token BTC
```

**Output (JSON):**
```json
{
  "ok": true,
  "token": "ETH",
  "price_usd": "2327.01",
  "open_24h": "2310.50",
  "high_24h": "2415.00",
  "low_24h": "2295.30",
  "volume_24h": "441282371.62",
  "change_24h_pct": "0.72",
  "source": "okx-public-api"
}
```

## Data Sources

- **OKX Public API**: `GET https://www.okx.com/api/v5/market/ticker?instId={TOKEN}-USDT`

## Safety

- Read-only plugin — no transactions, no wallet access
- No API keys required (uses public endpoints only)
- No sensitive data stored or transmitted
