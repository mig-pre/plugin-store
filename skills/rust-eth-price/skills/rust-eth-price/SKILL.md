---
name: rust-eth-price
description: "Query real-time ETH price via onchainos CLI"
version: "1.0.0"
author: "yz06276"
tags:
  - ethereum
  - price
---

# rust-eth-price

## Overview

A Rust CLI plugin that queries the current ETH price using the OKX API. Returns price in USD with 24h change data.

## Pre-flight Checks

Before using this skill, ensure:

1. The `onchainos` CLI is installed and configured
2. The `rust-eth-price` binary is installed

## Commands

### Get ETH Price

```bash
rust-eth-price --chain ethereum
```

**When to use**: When the user asks for the current price of ETH.
**Output**: Current ETH price in USD, 24h change percentage.

### Get ETH Price via onchainos

```bash
onchainos market price --token ETH --chain ethereum
```

**When to use**: Alternative method using onchainos directly.
**Output**: Current price, 24h volume, market cap.

## Error Handling

| Error | Cause | Resolution |
|-------|-------|------------|
| "Network error" | No internet connection | Check connectivity and retry |
| "Token not found" | Invalid token symbol | Verify token name |
| "Rate limited" | Too many requests | Wait 10 seconds and retry once |
