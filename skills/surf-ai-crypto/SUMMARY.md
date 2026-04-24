# surf-ai-crypto

## Overview

Surf is a natural-language crypto data layer for AI agents. One CLI gives the agent access to 83+ commands across 14 data domains — prices, markets, wallets, tokens, social intelligence, DeFi positions, on-chain SQL, prediction markets, perpetual futures, news, and more — spanning 40+ blockchains and 200+ data sources.

Core operations:

- Query real-time prices, order books, K-lines, funding rates, technical indicators
- Profile any wallet — balances, PnL, labels, transactions, DeFi positions, perp trades
- Track smart money — netflow, holdings, leaderboards, DCA strategies, copy-trade signals
- Analyze tokens — holders, flows, unlocks, DEX trades, holder clusters, social mindshare
- Query Polymarket prediction markets and Hyperliquid perp markets
- Read news and KOL signals with AI-rated sentiment and trade-actionability scores
- Run ad-hoc on-chain SQL against a unified multi-chain data warehouse

Tags: `crypto` `analytics` `onchain` `defi` `wallet` `social` `prediction-market`

## Prerequisites

- No IP restrictions for data queries (read-only)
- Supported chains: 40+ including Ethereum, Solana, Base, Arbitrum, Polygon, BSC, Avalanche, Optimism, Sui, Aptos, Hyperliquid
- Supported data: prices, order books, technicals, on-chain SQL, social signals, prediction markets, perp markets, NFT metadata
- Surf CLI installed — follow [agents.asksurf.ai/docs/cli/introduction](https://agents.asksurf.ai/docs/cli/introduction)
- Free tier: 30 credits per day, no API key required to start
- Full access: sign up at [agents.asksurf.ai](https://agents.asksurf.ai) and run `surf auth --api-key <key>` in your terminal (never paste keys into chat)

## Quick Start

1. **Install the Surf CLI**: follow the guide at [agents.asksurf.ai/docs/cli/introduction](https://agents.asksurf.ai/docs/cli/introduction), then run `surf install && surf sync` to pull the latest operations catalog.

2. **Verify**: run `surf list-operations` to see all available commands.

3. **Ask naturally**: tell the agent what crypto data you need and it will map to the right command. Examples: "what's the price of ETH?", "profile wallet 0xabc...", "show top 10 AAVE holders", "Polymarket odds for the next US election", "BTC funding rates across exchanges", "TVL of Uniswap on Arbitrum".

4. **(Optional) Authenticate for full access**: sign up at [agents.asksurf.ai](https://agents.asksurf.ai), copy your API key, then in your own terminal (not the chat window) run `surf auth --api-key <your-key>`.

5. **Route to on-chain actions when ready**: surf is read-only. For swaps, transfers, or DeFi positions, route to OKX Onchain OS skills — `okx-dex-swap` for token trades, `okx-agentic-wallet` for wallet actions, `okx-defi-invest` for DeFi deposits/withdraws.
