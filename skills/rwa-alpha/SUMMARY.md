# rwa-alpha
Real World Asset intelligence trading skill that detects macro events (Fed, CPI, gold, SEC rulings) via NewsNow headlines + Polymarket probability confirmation, then auto-trades 15 tokenized treasury/gold/yield/governance tokens via OKX DEX with dual exit systems.

## Highlights
- 3-layer macro event detection: keyword regex → LLM confirm (Haiku, ~$0.005/call) → LLM discover missed headlines
- 15 RWA tokens across 7 categories: treasury (USDY, OUSG, sDAI, bIB01), gold (PAXG, XAUT), DeFi yield (USDe), governance (ONDO, CFG, MPL, PENDLE, PLUME, OM, GFI, TRU)
- 15 macro event playbook entries with token-specific actions and conviction scores
- 3 strategy modes: Yield Optimizer (asset-backed only) / Macro Trader (balanced) / Full Alpha (all strategies)
- Dual exit system: NAV premium/discount for asset-backed tokens, TP/SL/trailing stop for governance tokens
- Multi-chain: Ethereum + Solana via Agentic Wallet TEE signing
- Composite sentiment scoring: 60% news + 40% on-chain
- Paper mode + PAUSED=True by default; zero pip dependencies
