# starter-coach

## Overview

Starter Coach is a conversational 6-step skill that guides any user — beginner or experienced — to design, backtest, paper trade, and deploy their own automated DEX spot-trading bot on OKX DEX. It asks the right questions, builds a validated strategy spec, runs a backtest, gates live mode behind a paper trading requirement, and requires explicit confirmation before any real funds are used. All on-chain execution is powered by OnchainOS Agentic Wallet with TEE signing — no API key needed.

## Prerequisites

- OnchainOS CLI installed (`onchainos --version`)
- OKX Agentic Wallet logged in (`onchainos wallet status`)
- Python 3.8+
- Sufficient balance on your chosen chain for trading

## Quick Start

1. Install the skill:
   ```bash
   plugin-store install starter-coach
   ```
2. Start the coach:
   ```
   Start starter coach
   ```
3. Follow the 6-step flow: **Onboard → Profile → Build Strategy → Backtest → Paper Trade → Go Live**
4. When ready to go live, type `CONFIRM` to activate live mode
