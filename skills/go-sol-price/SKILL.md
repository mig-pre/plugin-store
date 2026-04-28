---
name: go-sol-price
description: Query the current SOL/USDT spot price from OKX's public ticker API. Trigger when the user asks "SOL price", "Solana price", "查 SOL 价格", or wants a quick on-demand spot quote without an API key.
version: "1.0.0"
---

# go-sol-price

A minimal Go CLI that prints the current `SOL/USDT` spot price by hitting OKX's public ticker endpoint. No API key, no third-party Go dependencies — single self-contained binary, ~3 MB.

## When to use

Say one of these to trigger:
- "what's the SOL price"
- "Solana current price"
- "查 SOL 现价"
- "SOL/USDT 报价"

## Usage

```bash
go-sol-price                # → SOL/USDT: 178.42
go-sol-price --version      # → go-sol-price 1.0.0
go-sol-price --help
```

## How it works

Single `GET https://www.okx.com/api/v5/market/ticker?instId=SOL-USDT`, parses `data[0].last`, prints. 10-second timeout, exits non-zero on network or API error.

## Limitations

- Read-only — no trading, no signing, no on-chain action
- Single instrument hardcoded (`SOL-USDT`) — extend to other pairs if needed
- Public endpoint, no auth, subject to OKX rate limits (generous for public API)
