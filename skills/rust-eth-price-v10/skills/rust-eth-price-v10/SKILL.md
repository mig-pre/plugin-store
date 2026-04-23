---
name: rust-eth-price-v10
description: "Query ETH price"
version: "10.0.0"
author: "yz06276"
tags: [ethereum, price]
---
# rust-eth-price-v10
## Overview
Query ETH price from OKX API.
## Pre-flight Checks
1. onchainos CLI installed
## Commands
### Get ETH Price
```bash
rust-eth-price-v10
```
**When to use**: When user asks for ETH price.
## Error Handling
| Error | Cause | Resolution |
|-------|-------|------------|
| "Network error" | No internet | Retry |
