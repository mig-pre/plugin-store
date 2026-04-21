# Market Structure Analyzer

Crypto market-structure research agent that delivers institutional-grade analysis using free public APIs. Covers derivatives positioning, options flow (gamma wall, 25-delta skew), on-chain metrics (MVRV, realized price), exchange flows (Dune Analytics), and macro sentiment — the same data Glassnode charges $49-999/month for.

## Features

- **20 Real-Time Indicators** — funding rates, OI, basis, taker volume, long/short ratios, liquidation pressure, realized volatility, Fear & Greed, BTC dominance, stablecoin dry powder
- **Options Quant** — gamma wall (market-maker support/resistance), 25-delta skew (risk reversal), ATM IV, butterfly spread
- **On-Chain** — MVRV ratio + realized price from CoinMetrics (free, no API key)
- **Dune Analytics Exchange Flows** — ETH/stablecoin CEX net flows, per-exchange breakdown, whale transfer classification
- **Interactive Dashboard** — dark-themed HTML dashboard with all metrics, auto-generated per analysis
- **Multi-Token** — BTC, ETH (Tier 1, full 24 indicators), SOL/BNB/DOGE/ARB (Tier 2), any OKX-listed token (Tier 3)
- **Zero Dependencies** — Python stdlib only, no pip install needed

## Install

```bash
npx skills add okx/plugin-store --skill market-structure-analyzer
```

## Data Sources

| Source | Data | Cost |
|--------|------|------|
| OKX | Derivatives, options, taker volume, basis | Free |
| Binance | Funding fallback, L/S ratios, liquidation proxy | Free |
| CoinMetrics | MVRV, realized price (30d history) | Free |
| CoinGecko | BTC dominance, total market cap | Free |
| Alternative.me | Fear & Greed Index | Free |
| DefiLlama | Stablecoin market cap | Free |
| Dune Analytics | Exchange flows, whale transfers | Free (via MCP) |

## Risk Warning

> This tool provides market data and analysis for informational purposes only. It is NOT financial or trading advice. Always verify data with primary sources and do your own research before making any trading decisions.

## License

MIT
