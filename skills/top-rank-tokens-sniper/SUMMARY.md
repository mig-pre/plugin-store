## Overview

Top Rank Tokens Sniper is a Solana ranking-leaderboard sniper that scans the OKX 1-hour gainers Top 20 every 10 seconds, snipes a token on its first leaderboard appearance after passing 13 Slot Guard pre-checks + 9 Advanced Safety checks + 3 Holder Risk checks, and ranks candidates with a 0–125 momentum score (buy ratio, price change, active traders, liquidity). Positions are managed by a 6-layer exit system whose highest-priority rule is "rank-out" — auto-sell 100% when the token drops out of Top 20. All trading runs through the onchainos CLI Agentic Wallet (TEE signing — no private key handling) and is observed via a Web Dashboard at http://localhost:3244.

## Prerequisites
- onchainos CLI ≥ 2.0.0-beta installed and on PATH (`onchainos --version`)
- Agentic Wallet logged in (`onchainos wallet login <email>` → `onchainos wallet status`)
- Solana address ready for funding (`onchainos wallet addresses --chain 501`)
- Python 3 (standard library only — no `pip install` required)
- Start in Paper Mode (`MODE = "paper"`) before switching to Live Trading

## Quick Start
1. Install the skill: `npx skills add okx/plugin-store --skill top-rank-tokens-sniper`
2. Verify prerequisites: `onchainos --version` and `onchainos wallet status`
3. Optionally edit `config.py` to pick a risk preference (Conservative / Default / Aggressive) and tune `BUY_AMOUNT`, `TOTAL_BUDGET`, `MAX_POSITIONS`, `TP_TIERS`, `STOP_LOSS_PCT`, `QUICK_STOP_MIN`, `QUICK_STOP_PCT`, `TRAILING_ACTIVATE`, `TRAILING_DROP`, `MAX_HOLD_HOURS` — every parameter is annotated in the file
4. Confirm `MODE = "paper"` in `config.py` for the first run (no real funds are spent in Paper Mode)
5. Start the bot: `python3 ranking_sniper.py`
6. Open the Dashboard: `open http://localhost:3244`
7. After verifying signals look healthy, set `PAUSED = False` to allow new positions; to go Live, set `MODE = "live"` and re-confirm budget / per-trade size
8. Stop anytime: `pkill -f ranking_sniper.py`
