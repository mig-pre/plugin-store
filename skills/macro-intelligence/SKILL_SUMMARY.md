# macro-intelligence — Skill Summary

## Overview
Macro Intelligence is a unified macro news and sentiment aggregator that polls 7 data sources — NewsNow headlines, Polymarket prediction markets, 6551.io OpenNews (WebSocket + REST fallback), Finnhub market news with stock/ETF quotes (SPY, GLD, SLV), Telegram groups (30+ macro/whale/alpha channels), FRED economic indicators (Fed Funds Rate, CPI, GDP, Unemployment, yield curve), and the Crypto Fear & Greed Index. Each incoming message passes through a 3-layer classification pipeline: first keyword regex (24+ bilingual EN/CN patterns covering Fed rate decisions, CPI, gold, geopolitical events, tariffs, whale alerts, RWA catalysts), then LLM confirmation for ambiguous matches (0.55-0.80 confidence), then LLM discovery for relevant messages that missed all keywords. A macro playbook maps each classified event to direction, magnitude, and urgency. The Token Impact Engine maps each signal to specific crypto tokens with directional impact scores across 23 event types (e.g. `fed_cut_surprise -> BTC +0.85, ETH +0.80`), with client-side fallback for legacy signals. Source diversity logic guarantees minimum 5 signals per source type across 80 returned signals, preventing any single source from flooding the feed. Sender reputation tracks source reliability with 30-day decay and boosts high-rep senders 1.3x. Cross-source dedup uses MD5 hashing within a 4-hour window. The skill exposes 11 HTTP API endpoints for filtered signals, aggregate sentiment, market regime, Polymarket odds, FRED data, price tickers, and reputation leaderboards. No trading logic — downstream skills consume the signals. Dashboard at `http://localhost:3252`.

## Usage
Start with `python3 macro_news.py` — the skill begins polling all configured sources immediately. All sources are optional and degrade gracefully: without API keys, the skill still runs on NewsNow and Fear & Greed. Add Finnhub (`FINNHUB_API_KEY`), FRED (`FRED_API_KEY`), OpenNews (`OPENNEWS_TOKEN`), or Telegram (`TG_API_ID` / `TG_API_HASH`) for richer coverage. LLM classification requires `ANTHROPIC_API_KEY` (uses Claude Haiku). State persists to `state/state.json` every 10 seconds. Prerequisites: onchainos CLI >= 2.1.0, Python >= 3.9.

## Commands
| Command | Description |
|---|---|
| `python3 macro_news.py` | Start the intelligence feed + dashboard |
| `onchainos wallet login` | Authenticate (required for onchainos integration) |

## Triggers
Activates when the user mentions macro intelligence, macro news feed, sentiment aggregator, macro-intelligence, Fed/CPI/macro signals, news classification, real-time macro monitoring for trading, token impact, or source diversity.
