---
name: py-eth-price
description: "Query real-time ETH price via OKX API"
version: "1.0.0"
author: "yz06276"
tags:
  - ethereum
  - price
---

# py-eth-price

## Overview

A Python script that queries the current ETH price from the OKX public API.

## Pre-flight Checks

Before using this skill, ensure:

1. Python 3 is installed
2. The `onchainos` CLI is installed and configured

## Commands

### Get ETH Price

```bash
python3 src/main.py
```

**When to use**: When the user asks for the current price of ETH.
**Output**: JSON with price, 24h change, and volume.

### Get ETH Price via onchainos

```bash
onchainos market price --token ETH --chain ethereum
```

**When to use**: Alternative method using onchainos directly.

## Error Handling

| Error | Cause | Resolution |
|-------|-------|------------|
| "Connection error" | No internet | Check connectivity and retry |
| "Rate limited" | Too many requests | Wait 10 seconds and retry |
