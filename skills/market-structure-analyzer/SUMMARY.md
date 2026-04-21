# market-structure-analyzer
Crypto market-structure research agent delivering institutional-grade analysis
with 24 indicators across derivatives, options, on-chain, exchange flows, and
macro sentiment — all from free public APIs, zero cost.

## Highlights
- 20 real-time indicators: funding, OI, basis, taker volume, L/S ratios, liquidation pressure, realized vol, Fear & Greed, BTC dominance, stablecoin dry powder
- Options quant (Tier 1): gamma wall, 25-delta skew, ATM IV, butterfly spread — from OKX opt-summary
- On-chain: MVRV ratio + realized price from CoinMetrics free API (BTC + ETH)
- Dune Analytics: exchange flows, whale transfers, stablecoin CEX flows (4 pre-built queries)
- Interactive dark-themed HTML dashboard auto-generated per analysis
- Multi-token support: BTC/ETH (24 indicators), SOL/BNB/DOGE/ARB (Tier 2), any OKX-listed (Tier 3)
- Python stdlib only — zero pip dependencies
