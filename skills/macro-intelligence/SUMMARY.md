# macro-intelligence

## Overview

Unified macro intelligence feed that reads 7 sources (NewsNow, Polymarket, OpenNews, Finnhub, Telegram, FRED, Fear & Greed), classifies events with regex + LLM, scores sentiment, and exposes filtered signals via HTTP API for downstream trading skills. Read-only intelligence feed — no trading logic.

Core operations:

- Aggregate financial news from 7 sources with automatic polling
- Classify events into 24+ types with bilingual pattern matching and LLM confirmation
- Map macro signals to specific crypto tokens with directional impact scores
- Expose 11 HTTP API endpoints for filtered signals, sentiment, and regime detection
- Render neon-glass terminal dashboard with heat columns, sparklines, and token impact pills

Tags: `macro` `news` `sentiment` `intelligence` `fred` `polymarket` `dashboard`

## Prerequisites

- Python 3.8+ (stdlib only, no pip dependencies)
- Optional: Finnhub API key (`FINNHUB_API_KEY`), Telegram bot token (`TELEGRAM_BOT_TOKEN`) — sources degrade gracefully
- Optional: `ANTHROPIC_API_KEY` for LLM-powered headline classification

## Quick Start

1. **Start the server**: Run `python3 macro_news.py` from the skill directory. Dashboard opens at `http://localhost:3250` and API at `http://localhost:3250/api/signals`.

2. **Browse signals**: The dashboard shows a live feed of classified macro events with sentiment scores, source attribution, and token impact predictions. All 7 sources poll automatically.

3. **Query the API**: Use endpoints like `/api/signals?type=fed_rate` to filter by event type, `/api/sentiment` for aggregate sentiment, or `/api/regime` for current market regime detection.

4. **Integrate with trading skills**: Downstream skills (like rwa-alpha) can poll the signals API to trigger trades based on macro events.
