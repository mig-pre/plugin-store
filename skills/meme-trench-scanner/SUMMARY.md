## Overview

Meme Trench Scanner is a Solana meme-token trading bot that scans 11 Launchpads (pump.fun, Believe, LetsBonk, …) every 10 seconds, detects entries via TX acceleration + volume surge + 5m/15m buy-sell ratio, runs deep safety checks (dev rug history, bundler holdings, LP lock, aped wallets), and manages exits through a 7-layer system (emergency exit, FAST_DUMP crash detection, stop loss, trailing stop, tiered TP). All trading is driven by the onchainos CLI Agentic Wallet (TEE signing — no private key handling) and observed via a Web Dashboard at http://localhost:3241.

## Prerequisites
- onchainos CLI ≥ 2.1.0 installed and on PATH (`onchainos --version`)
- Agentic Wallet logged in (`onchainos wallet login <email>` → `onchainos wallet status`)
- Solana address ready for funding (`onchainos wallet addresses --chain 501`)
- Python 3 (standard library only — no `pip install` required)
- Start in Paper Mode (`PAPER_TRADE = True`) before considering Live Trading

## Quick Start
1. Install the skill: `npx skills add okx/plugin-store --skill meme-trench-scanner`
2. Verify prerequisites: `onchainos --version` and `onchainos wallet status`
3. Optionally edit `config.py` to pick a risk preference (Conservative / Default / Aggressive) and tune `MAX_SOL`, `SOL_PER_TRADE`, `TP1_PCT`, `TP2_PCT`, `S1_PCT`, `MAX_POSITIONS`, `MAX_HOLD_MIN` — every parameter is annotated in the file
4. Confirm `PAPER_TRADE = True` in `config.py` for the first run (no real funds are spent in Paper Mode)
5. Start the bot: `python3 scan_live.py` (or background: `nohup python3 scan_live.py > bot.log 2>&1 &`)
6. Open the Dashboard: `open http://localhost:3241`
7. After verifying signals look healthy, set `PAUSED = False` in `config.py` to allow new positions; to go Live, set `PAPER_TRADE = False` and re-confirm exposure parameters
8. Stop anytime: `pkill -f scan_live.py`
