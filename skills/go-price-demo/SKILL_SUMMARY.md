
# go-price-demo -- Skill Summary

## Overview
This is a demonstration plugin built in Go that provides real-time cryptocurrency price data by querying the OKX public API. It supports fetching price information for various tokens including ETH, BTC, and SOL, returning comprehensive market data including current price, 24-hour statistics, and volume information in a structured JSON format.

## Usage
Run `go-price-demo get-price --token <SYMBOL>` to fetch current price data for a specified cryptocurrency token. The default token is ETH if no symbol is provided.

## Commands
| Command | Description | Parameters |
|---------|-------------|------------|
| `get-price` | Get current token price and 24h market data | `--token <SYMBOL>` (default: ETH) |

## Triggers
An AI agent should activate this skill when users request current cryptocurrency prices, market data, or 24-hour trading statistics for supported tokens like ETH, BTC, or SOL.
