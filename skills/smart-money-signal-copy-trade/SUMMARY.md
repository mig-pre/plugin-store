## Overview

Smart Money Signal Copy Trade is a Solana copy-trading bot that polls OKX Smart Money / KOL / Whale buy signals every 20 seconds, triggers an entry only when ≥3 tracked wallets buy the same token simultaneously (co-rider consensus), runs 15 safety filters (market cap, liquidity, holders, dev rug, bundler, LP burn, K1 pump, …), and manages exits through a 7-layer system (cost-aware TP1/TP2/TP3, hard stop, time-decay SL, trailing stop, trend stop, liquidity emergency). All trading runs through the onchainos CLI Agentic Wallet (TEE signing — no private key handling); `config.py` supports hot-reload, and a Web Dashboard is exposed at http://localhost:3248.

## Prerequisites
- onchainos CLI ≥ 2.0.0-beta installed and on PATH (`onchainos --version`)
- Agentic Wallet logged in (`onchainos wallet login <email>` → `onchainos wallet status`)
- Solana address ready for funding (`onchainos wallet addresses --chain 501`)
- Python 3 (standard library only — no `pip install` required)
- Start in Paper Mode (`DRY_RUN = True`) before switching to Live Mode

## Quick Start
1. Install the skill: `npx skills add okx/plugin-store --skill smart-money-signal-copy-trade`
2. Verify prerequisites: `onchainos --version` and `onchainos wallet status`
3. Optionally edit `config.py` to pick a risk preference (Conservative / Default / Aggressive) and tune `POSITION_TIERS`, `MAX_POSITIONS`, `MIN_WALLET_COUNT`, `TP_TIERS`, `SL_MULTIPLIER`, `TIME_DECAY_SL`, `TRAIL_ACTIVATE`, `TRAIL_DISTANCE`, `TIME_STOP_MAX_HOLD_HRS` — every parameter is annotated, and changes hot-reload
4. Confirm `DRY_RUN = True` in `config.py` for the first run (no real funds are spent in Paper Mode)
5. Start the bot: `python3 bot.py`
6. Open the Dashboard: `open http://localhost:3248`
7. After verifying signals look healthy, set `PAUSED = False` to allow new positions; to go Live, set `DRY_RUN = False` and re-confirm budget / loss limits
8. Stop anytime: `pkill -f bot.py`
