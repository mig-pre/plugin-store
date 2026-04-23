# macro-intelligence

## Overview
Unified macro intelligence feed that reads 7 sources (NewsNow, Polymarket, OpenNews, Finnhub, Telegram, FRED, Fear & Greed), classifies events with regex + LLM, scores sentiment, and exposes filtered signals via HTTP API for downstream trading skills. Features 24+ event types with bilingual patterns, a Token Impact Engine mapping macro signals to crypto tokens, and a neon-glass terminal dashboard. Read-only intelligence feed — no trading logic.

## Prerequisites
- Python 3.8+
- No pip dependencies required (Python stdlib only)
- Optional: Finnhub API key (`FINNHUB_API_KEY`), Telegram bot token (`TELEGRAM_BOT_TOKEN`) — sources degrade gracefully without API keys
- Optional: `ANTHROPIC_API_KEY` for LLM-powered headline classification

## Quick Start
```bash
# Start the macro intelligence server
cd skills/macro-intelligence
python3 macro_news.py
# → Dashboard at http://localhost:3250
# → API at http://localhost:3250/api/signals
```

Provides 11 HTTP API endpoints for filtered signals, sentiment, regime detection, Polymarket probabilities, and price tickers. All 7 data sources poll automatically with configurable intervals.
