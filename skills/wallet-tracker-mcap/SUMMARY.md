# wallet-tracker-mcap
Wallet copy-trade bot monitoring target Solana wallets for meme token trades with auto buy/sell mirroring, MC target gating, and 4-tier risk grading.

## Highlights
- Two follow modes: MC_TARGET (wait for market cap proof) or INSTANT (immediate follow)
- 5 exit triggers: mirror sell, stop loss, tiered take-profit, trailing stop, time stop
- 4-tier risk grading: honeypot, rug history, wash trading, liquidity drain detection
- Pre-trade safety gates: liquidity, holders, top10, dev hold, bundle checks
- Post-trade monitoring: active dump, LP drain, coordinated selling → auto exit
- onchainos Agentic Wallet TEE signing -- no private keys in code
- Paper Mode + PAUSED=True default -- safe to test
- Web dashboard at localhost:3248 with positions, watch list, trades, live feed
- Zero pip dependencies -- Python 3.8+ stdlib only + onchainos CLI
