# market-structure-analyzer -- Skill Summary

## Overview
Market Structure Analyzer is a crypto research agent that fetches, analyzes, and presents 24 institutional-grade indicators across derivatives, options, on-chain, exchange flows, and macro sentiment. All data comes from free public APIs (OKX, Binance, CoinMetrics, CoinGecko, DefiLlama, Alternative.me) plus optional Dune Analytics on-chain queries.

## Usage
The agent runs `python3 scripts/fetch_market_data.py BTC ETH` to collect real-time data, optionally executes 4 pre-built Dune queries for exchange flow analysis, then generates an interactive HTML dashboard and a structured chat analysis report.

## Commands
| Command | Description |
|---|---|
| `python3 scripts/fetch_market_data.py BTC` | Fetch all indicators for BTC |
| `python3 scripts/fetch_market_data.py BTC ETH SOL` | Multi-token fetch |
| Dune query 6988944 | ETH CEX net flows (7d) |
| Dune query 6988945 | CEX flows by exchange (24h) |
| Dune query 6988947 | Whale ETH transfers (24h) |
| Dune query 6988949 | Stablecoin CEX flows (7d) |

## Triggers
Activates when the user mentions market structure, derivatives analysis, gamma wall, options skew, funding rates, open interest, MVRV, exchange flows, whale tracking, fear and greed, macro overview, is the market overleveraged, is BTC about to move, what does the market look like, CEX inflows/outflows, stablecoin flows.
