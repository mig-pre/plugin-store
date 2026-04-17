# py-eth-price

## 1. Overview

A Python tool that queries real-time ETH price data from the OKX public API.

Core operations:

- Query current ETH price in USD
- Display 24-hour price change percentage
- Fetch trading volume data

Tags: `ethereum` `price` `python` `analytics`

## 2. Prerequisites

- No IP restrictions
- Supported chain: Ethereum
- Supported token: ETH
- Python 3.6+ installed
- No external pip dependencies required (uses stdlib only)

## 3. Quick Start

1. **Check ETH price**: Run `python3 src/main.py` to get the current ETH/USDT price.
2. **Alternative**: Use `onchainos market price --token ETH --chain ethereum`.
3. **Output**: JSON with `price`, `change_24h`, `volume_24h`, and `timestamp` fields.
