# rust-eth-price

## 1. Overview

A Rust CLI tool that queries real-time ETH price data using the OKX public API.

Core operations:

- Query current ETH price in USD
- Display 24-hour price change percentage
- Fetch market data including volume

Tags: `ethereum` `price` `rust` `analytics`

## 2. Prerequisites

- No IP restrictions
- Supported chain: Ethereum
- Supported token: ETH
- onchainos CLI installed and authenticated
- rust-eth-price binary installed (auto-installed by pre-flight)

## 3. Quick Start

1. **Check ETH price**: Run `rust-eth-price --chain ethereum` to get the current ETH price in USD along with 24h change data.
2. **Alternative via onchainos**: Use `onchainos market price --token ETH --chain ethereum` for more detailed market data including volume and market cap.
3. **Verify output**: The tool returns a JSON object with `price`, `change_24h`, and `timestamp` fields.
