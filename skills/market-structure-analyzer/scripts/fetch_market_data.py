#!/usr/bin/env python3
"""Market Structure Data Fetcher v2.0 — OKX-first, with Binance fallback.

Usage:
    python3 fetch_market_data.py BTC          # single token
    python3 fetch_market_data.py BTC ETH SOL  # multi-token
    python3 fetch_market_data.py --all         # all supported tokens

Outputs JSON to stdout with all available indicators per token.
Priority: OKX APIs first, Binance as fallback, then CoinGecko/DefiLlama for macro.

v2.0 changes:
  - Fixed OKX error response handling (check code != "0")
  - Fixed taker volume + long/short API params
  - Added safe float parsing
  - Added retry logic (1 retry with backoff)
  - Added: funding rate history (trend), OI delta, realized volatility,
    cross-exchange OI comparison, liquidation data, options open interest breakdown
"""
from __future__ import annotations

import json
import math
import sys
import time
import urllib.request
import urllib.parse
from datetime import datetime, timezone

# ── Config ──────────────────────────────────────────────────────────

TOKEN_MAP = {
    "BTC": {
        "okx_swap": "BTC-USDT-SWAP", "okx_spot": "BTC-USDT", "okx_family": "BTC-USD",
        "binance": "BTCUSDT", "coingecko": "bitcoin", "tier": 1,
    },
    "ETH": {
        "okx_swap": "ETH-USDT-SWAP", "okx_spot": "ETH-USDT", "okx_family": "ETH-USD",
        "binance": "ETHUSDT", "coingecko": "ethereum", "tier": 1,
    },
    "SOL": {
        "okx_swap": "SOL-USDT-SWAP", "okx_spot": "SOL-USDT", "okx_family": "SOL-USD",
        "binance": "SOLUSDT", "coingecko": "solana", "tier": 2,
    },
    "BNB": {
        "okx_swap": "BNB-USDT-SWAP", "okx_spot": "BNB-USDT", "okx_family": "",
        "binance": "BNBUSDT", "coingecko": "binancecoin", "tier": 2,
    },
    "DOGE": {
        "okx_swap": "DOGE-USDT-SWAP", "okx_spot": "DOGE-USDT", "okx_family": "",
        "binance": "DOGEUSDT", "coingecko": "dogecoin", "tier": 2,
    },
    "AVAX": {
        "okx_swap": "AVAX-USDT-SWAP", "okx_spot": "AVAX-USDT", "okx_family": "",
        "binance": "AVAXUSDT", "coingecko": "avalanche-2", "tier": 2,
    },
    "ARB": {
        "okx_swap": "ARB-USDT-SWAP", "okx_spot": "ARB-USDT", "okx_family": "",
        "binance": "ARBUSDT", "coingecko": "arbitrum", "tier": 2,
    },
    "XRP": {
        "okx_swap": "XRP-USDT-SWAP", "okx_spot": "XRP-USDT", "okx_family": "",
        "binance": "XRPUSDT", "coingecko": "ripple", "tier": 2,
    },
    "LINK": {
        "okx_swap": "LINK-USDT-SWAP", "okx_spot": "LINK-USDT", "okx_family": "",
        "binance": "LINKUSDT", "coingecko": "chainlink", "tier": 2,
    },
    "PEPE": {
        "okx_swap": "PEPE-USDT-SWAP", "okx_spot": "PEPE-USDT", "okx_family": "",
        "binance": "PEPEUSDT", "coingecko": "pepe", "tier": 3,
    },
}

OKX_BASE = "https://www.okx.com/api/v5"
BINANCE_FAPI = "https://fapi.binance.com"
BINANCE_SPOT = "https://api.binance.com/api/v3"
HEADERS = {"User-Agent": "okx-market-analyzer/2.0", "Accept": "application/json"}


# ── Helpers ─────────────────────────────────────────────────────────

def safe_float(val, default=0.0) -> float:
    """Safely convert to float; returns default on None, empty string, or bad value."""
    if val is None or val == "":
        return default
    try:
        return float(val)
    except (ValueError, TypeError):
        return default


def fetch(url: str, timeout: int = 12, retries: int = 1) -> dict | list | None:
    """Fetch JSON from URL with retry. Returns parsed data or error dict."""
    for attempt in range(retries + 1):
        try:
            req = urllib.request.Request(url, headers=HEADERS)
            with urllib.request.urlopen(req, timeout=timeout) as resp:
                return json.loads(resp.read().decode())
        except Exception as e:
            if attempt < retries:
                time.sleep(0.5 * (attempt + 1))
                continue
            return {"_error": str(e), "_url": url}


def is_error(data) -> bool:
    return data is None or (isinstance(data, dict) and "_error" in data)


def okx_data(resp) -> list:
    """Extract .data array from OKX API response, checking for OKX error codes."""
    if is_error(resp):
        return []
    if not isinstance(resp, dict):
        return []
    # OKX returns {"code": "0", "data": [...]} on success
    # and {"code": "50011", "data": [], "msg": "Rate limit"} on error
    code = resp.get("code", "0")
    if code != "0":
        return []
    return resp.get("data", [])


# ═══════════════════════════════════════════════════════════════════
# OKX APIs (PRIMARY)
# ═══════════════════════════════════════════════════════════════════

def okx_funding(inst_id: str) -> dict:
    """Current + next funding rate from OKX."""
    if not inst_id:
        return {"status": "unavailable"}
    items = okx_data(fetch(f"{OKX_BASE}/public/funding-rate?instId={inst_id}"))
    if not items:
        return {"status": "unavailable"}
    r = items[0]
    rate = safe_float(r.get("fundingRate"))
    next_rate = safe_float(r.get("nextFundingRate")) if r.get("nextFundingRate") else None
    return {
        "status": "available",
        "rate": rate,
        "rate_pct": round(rate * 100, 6),
        "rate_annualized_pct": round(rate * 3 * 365 * 100, 2),
        "next_rate": next_rate,
        "next_rate_pct": round(next_rate * 100, 6) if next_rate is not None else None,
        "time": r.get("fundingTime", ""),
        "source": "okx",
    }


def okx_funding_history(inst_id: str, limit: int = 6) -> dict:
    """Recent funding rate history from OKX (last N periods = 48h at 8h intervals).

    Returns trend direction and average rate.
    """
    if not inst_id:
        return {"status": "unavailable"}
    items = okx_data(fetch(f"{OKX_BASE}/public/funding-rate-history?instId={inst_id}&limit={limit}"))
    if not items:
        return {"status": "unavailable"}
    rates = [safe_float(r.get("fundingRate")) for r in items]
    if not rates:
        return {"status": "unavailable"}

    avg = sum(rates) / len(rates)
    # Trend: compare first half vs second half
    mid = len(rates) // 2
    recent_avg = sum(rates[:mid]) / max(mid, 1)  # items[0] is most recent
    older_avg = sum(rates[mid:]) / max(len(rates) - mid, 1)
    if recent_avg > older_avg * 1.2:
        trend = "increasing"
    elif recent_avg < older_avg * 0.8:
        trend = "decreasing"
    else:
        trend = "stable"

    return {
        "status": "available",
        "rates": [round(r * 100, 6) for r in rates],  # as pct
        "avg_rate_pct": round(avg * 100, 6),
        "avg_annualized_pct": round(avg * 3 * 365 * 100, 2),
        "trend": trend,
        "periods": len(rates),
        "source": "okx",
    }


def okx_open_interest(inst_id: str) -> dict:
    """Open interest from OKX."""
    if not inst_id:
        return {"status": "unavailable"}
    items = okx_data(fetch(f"{OKX_BASE}/public/open-interest?instType=SWAP&instId={inst_id}"))
    if not items:
        return {"status": "unavailable"}
    r = items[0]
    return {
        "status": "available",
        "oi": safe_float(r.get("oi")),
        "oi_currency": safe_float(r.get("oiCcy")),
        "source": "okx",
    }


def okx_oi_history(ccy: str) -> dict:
    """OI + volume history from OKX rubik (24h). Computes OI delta.

    OKX returns data as arrays: [ts, oi, vol] per item.
    """
    url = f"{OKX_BASE}/rubik/stat/contracts/open-interest-volume?ccy={ccy}&period=1D"
    items = okx_data(fetch(url))
    if not items or len(items) < 2:
        return {"status": "unavailable"}

    # items can be list-of-lists [ts, oi, vol] or list-of-dicts
    def extract_oi(item):
        if isinstance(item, list) and len(item) >= 2:
            return safe_float(item[1])
        elif isinstance(item, dict):
            return safe_float(item.get("oi"))
        return 0.0

    latest_oi = extract_oi(items[0])
    prev_oi = extract_oi(items[1]) if len(items) > 1 else latest_oi
    day_ago_oi = extract_oi(items[-1])

    oi_delta_1d_pct = ((latest_oi - day_ago_oi) / day_ago_oi * 100) if day_ago_oi > 0 else 0
    oi_delta_step_pct = ((latest_oi - prev_oi) / prev_oi * 100) if prev_oi > 0 else 0

    return {
        "status": "available",
        "latest_oi": latest_oi,
        "oi_delta_1d_pct": round(oi_delta_1d_pct, 2),
        "oi_delta_step_pct": round(oi_delta_step_pct, 2),
        "data_points": len(items),
        "source": "okx",
    }


def okx_ticker(inst_id: str) -> dict:
    """24h ticker from OKX spot."""
    if not inst_id:
        return {"status": "unavailable"}
    items = okx_data(fetch(f"{OKX_BASE}/market/ticker?instId={inst_id}"))
    if not items:
        return {"status": "unavailable"}
    r = items[0]
    last = safe_float(r.get("last"))
    open24 = safe_float(r.get("open24h"))
    change_pct = ((last - open24) / open24 * 100) if open24 > 0 else 0
    return {
        "status": "available",
        "price": last,
        "open_24h": open24,
        "high_24h": safe_float(r.get("high24h")),
        "low_24h": safe_float(r.get("low24h")),
        "volume_24h_base": safe_float(r.get("vol24h")),
        "volume_24h_quote": safe_float(r.get("volCcy24h")),
        "price_change_pct": round(change_pct, 2),
        "source": "okx",
    }


def okx_candle_history(inst_id: str, bar: str = "1H", limit: int = 24) -> list:
    """Fetch candle data for volatility calculation. Returns list of close prices."""
    if not inst_id:
        return []
    items = okx_data(fetch(f"{OKX_BASE}/market/candles?instId={inst_id}&bar={bar}&limit={limit}"))
    if not items:
        return []
    # OKX candle: [ts, open, high, low, close, vol, volCcy, volCcyQuote, confirm]
    closes = []
    for c in items:
        if isinstance(c, list) and len(c) >= 5:
            closes.append(safe_float(c[4]))
    return closes


def compute_realized_volatility(closes: list) -> dict:
    """Compute realized volatility from hourly close prices.

    Returns annualized vol (sqrt(8760) for hourly data).
    """
    if len(closes) < 3:
        return {"status": "unavailable"}

    # Log returns
    returns = []
    for i in range(1, len(closes)):
        if closes[i - 1] > 0 and closes[i] > 0:
            returns.append(math.log(closes[i] / closes[i - 1]))

    if len(returns) < 2:
        return {"status": "unavailable"}

    mean = sum(returns) / len(returns)
    variance = sum((r - mean) ** 2 for r in returns) / (len(returns) - 1)
    hourly_vol = math.sqrt(variance)
    annualized_vol = hourly_vol * math.sqrt(8760)  # 8760 hours/year

    return {
        "status": "available",
        "realized_vol_1h": round(hourly_vol * 100, 4),
        "realized_vol_annualized_pct": round(annualized_vol * 100, 2),
        "sample_hours": len(returns),
        "source": "okx",
    }


def okx_long_short(inst_id: str) -> dict:
    """Long/short account ratio from OKX. Uses full instId (e.g. BTC-USDT-SWAP)."""
    if not inst_id:
        return {"status": "unavailable"}
    # OKX rubik contract-player expects the currency, not the full instId
    ccy = inst_id.split("-")[0]
    url = f"{OKX_BASE}/rubik/stat/contracts/long-short-account-ratio/contract-player?instId={ccy}&period=1H"
    items = okx_data(fetch(url))
    if not items:
        return {"status": "unavailable"}
    r = items[0]
    return {
        "status": "available",
        "long_ratio": safe_float(r.get("longRatio")) if r.get("longRatio") else None,
        "short_ratio": safe_float(r.get("shortRatio")) if r.get("shortRatio") else None,
        "source": "okx",
    }


def okx_options_summary(inst_family: str) -> dict:
    """Options summary: put/call ratio, max pain from OKX."""
    if not inst_family:
        return {"status": "unavailable", "reason": "No options market for this token"}
    items = okx_data(fetch(f"{OKX_BASE}/public/opt-summary?instFamily={inst_family}"))
    if not items:
        return {"status": "unavailable"}
    r = items[0]
    return {
        "status": "available",
        "put_call_ratio": safe_float(r.get("putCallRatio")) or None,
        "max_pain": safe_float(r.get("maxPain")) or None,
        "call_volume": safe_float(r.get("callVol")) or None,
        "put_volume": safe_float(r.get("putVol")) or None,
        "call_oi": safe_float(r.get("callOi")) or None,
        "put_oi": safe_float(r.get("putOi")) or None,
        "source": "okx",
    }


def okx_taker_volume(ccy: str) -> dict:
    """Taker buy/sell volume ratio from OKX contracts.

    Endpoint: rubik/stat/taker-volume?ccy=BTC&instType=CONTRACTS
    Returns arrays: [ts, sellVol, buyVol] — note: sell first, buy second.
    """
    if not ccy:
        return {"status": "unavailable"}
    url = f"{OKX_BASE}/rubik/stat/taker-volume?ccy={ccy}&instType=CONTRACTS&period=1H"
    items = okx_data(fetch(url))
    if not items:
        return {"status": "unavailable"}
    r = items[0]
    if isinstance(r, list) and len(r) >= 3:
        sell_vol = safe_float(r[1])  # index 1 = sellVol
        buy_vol = safe_float(r[2])   # index 2 = buyVol
    elif isinstance(r, dict):
        buy_vol = safe_float(r.get("buyVol"))
        sell_vol = safe_float(r.get("sellVol"))
    else:
        return {"status": "unavailable"}

    ratio = (buy_vol / sell_vol) if sell_vol > 0 else None
    return {
        "status": "available",
        "buy_volume": round(buy_vol, 2),
        "sell_volume": round(sell_vol, 2),
        "buy_sell_ratio": round(ratio, 4) if ratio else None,
        "interpretation": (
            "aggressive buying" if ratio and ratio > 1.1
            else "aggressive selling" if ratio and ratio < 0.9
            else "balanced"
        ),
        "source": "okx",
    }


def okx_futures_basis(swap_id: str, spot_id: str) -> dict:
    """Futures basis (premium) — OKX swap price vs spot."""
    swap_items = okx_data(fetch(f"{OKX_BASE}/market/ticker?instId={swap_id}"))
    spot_items = okx_data(fetch(f"{OKX_BASE}/market/ticker?instId={spot_id}"))
    if not swap_items or not spot_items:
        return {"status": "unavailable"}
    swap_price = safe_float(swap_items[0].get("last"))
    spot_price = safe_float(spot_items[0].get("last"))
    if spot_price == 0:
        return {"status": "unavailable"}
    basis_pct = ((swap_price - spot_price) / spot_price) * 100
    return {
        "status": "available",
        "swap_price": swap_price,
        "spot_price": spot_price,
        "basis_pct": round(basis_pct, 4),
        "interpretation": (
            "contango (longs paying premium)" if basis_pct > 0.01
            else "backwardation (shorts paying premium)" if basis_pct < -0.01
            else "near parity"
        ),
        "source": "okx",
    }


# ═══════════════════════════════════════════════════════════════════
# ON-CHAIN: MVRV + REALIZED PRICE (CoinMetrics free API)
# ═══════════════════════════════════════════════════════════════════

def coinmetrics_mvrv(asset: str, spot_price: float = 0) -> dict:
    """Fetch MVRV ratio from CoinMetrics community API (free, no key).

    MVRV = Market Cap / Realized Cap.
    >3.5 = historically overheated. <1.0 = undervalued (holders underwater).
    """
    cm_asset = {"BTC": "btc", "ETH": "eth"}.get(asset.upper(), "")
    if not cm_asset:
        return {"status": "unavailable", "reason": "MVRV only available for BTC/ETH"}

    end = datetime.now(timezone.utc)
    start_str = (end - __import__("datetime").timedelta(days=30)).strftime("%Y-%m-%dT00:00:00Z")

    url = (
        f"https://community-api.coinmetrics.io/v4/timeseries/asset-metrics"
        f"?assets={cm_asset}&metrics=CapMVRVCur&frequency=1d"
        f"&start_time={start_str}&page_size=60"
    )
    data = fetch(url, timeout=15)
    if is_error(data):
        return {"status": "unavailable"}

    items = data.get("data", [])
    if not items:
        return {"status": "unavailable"}

    # Filter to this asset
    values = [safe_float(i.get("CapMVRVCur")) for i in items if i.get("asset") == cm_asset and i.get("CapMVRVCur")]
    if not values:
        return {"status": "unavailable"}

    latest = values[-1]
    high_30d = max(values)
    low_30d = min(values)
    avg_30d = sum(values) / len(values)

    # Realized price = spot / MVRV
    realized_price = round(spot_price / latest, 2) if latest > 0 and spot_price > 0 else None

    # Interpretation
    if latest > 3.5:
        zone = "overheated (historically top territory)"
    elif latest > 2.5:
        zone = "elevated (caution)"
    elif latest > 1.5:
        zone = "healthy"
    elif latest > 1.0:
        zone = "undervalued (accumulation zone)"
    else:
        zone = "deep value (holders underwater)"

    return {
        "status": "available",
        "mvrv": round(latest, 4),
        "mvrv_30d_high": round(high_30d, 4),
        "mvrv_30d_low": round(low_30d, 4),
        "mvrv_30d_avg": round(avg_30d, 4),
        "realized_price": realized_price,
        "zone": zone,
        "data_points": len(values),
        "source": "coinmetrics",
    }


# ═══════════════════════════════════════════════════════════════════
# OPTIONS: GAMMA WALL + SKEW (OKX option chain)
# ═══════════════════════════════════════════════════════════════════

def okx_gamma_wall(inst_family: str, spot_price: float = 0) -> dict:
    """Compute gamma exposure by strike from OKX option chain.

    Gamma wall = strike with largest net gamma × OI. Market makers who are
    net short options have negative gamma — price tends to stick near high-gamma
    strikes (pin risk) and accelerate away from low-gamma zones.
    """
    if not inst_family:
        return {"status": "unavailable", "reason": "No options market"}

    # Fetch opt-summary (has Greeks: gammaBS, deltaBS, markVol per instrument)
    summary_data = fetch(f"{OKX_BASE}/public/opt-summary?instFamily={inst_family}", timeout=15)
    summary_items = okx_data(summary_data) if not is_error(summary_data) else []

    # Fetch per-instrument OI
    oi_data = fetch(f"{OKX_BASE}/public/open-interest?instType=OPTION&instFamily={inst_family}", timeout=15)
    oi_items = okx_data(oi_data) if not is_error(oi_data) else []

    if not summary_items or not oi_items:
        return {"status": "unavailable"}

    # Build OI map: instId → oiCcy
    oi_map = {}
    for item in oi_items:
        inst_id = item.get("instId", "")
        oi_map[inst_id] = safe_float(item.get("oiCcy"))

    # Build gamma map: aggregate gamma × OI by strike
    from collections import defaultdict
    gamma_by_strike = defaultdict(float)
    call_oi_by_strike = defaultdict(float)
    put_oi_by_strike = defaultdict(float)

    for t in summary_items:
        inst_id = t.get("instId", "")
        parts = inst_id.split("-")
        if len(parts) != 5:
            continue

        strike = safe_float(parts[3])
        cp = parts[4]  # C or P
        gamma_bs = safe_float(t.get("gammaBS"))
        oi = oi_map.get(inst_id, 0)

        if oi <= 0 or gamma_bs <= 0:
            continue

        # Net gamma exposure (MM perspective: short options → negative gamma)
        # Calls: MM short gamma if calls are bought
        # Puts: MM short gamma if puts are bought
        # For gamma wall: just aggregate |gamma × OI| per strike
        gamma_exposure = gamma_bs * oi * spot_price * spot_price * 0.01 if spot_price > 0 else gamma_bs * oi
        gamma_by_strike[strike] += gamma_exposure

        if cp == "C":
            call_oi_by_strike[strike] += oi
        else:
            put_oi_by_strike[strike] += oi

    if not gamma_by_strike:
        return {"status": "unavailable"}

    # Sort by gamma exposure, find top 5 gamma walls
    sorted_strikes = sorted(gamma_by_strike.items(), key=lambda x: -x[1])
    top_5 = sorted_strikes[:5]

    # Find the biggest gamma wall
    wall_strike = top_5[0][0]
    wall_gamma = top_5[0][1]

    # Classify: above spot = resistance, below spot = support
    if spot_price > 0:
        wall_type = "resistance (price magnet above)" if wall_strike > spot_price else "support (price magnet below)"
    else:
        wall_type = "unknown"

    return {
        "status": "available",
        "gamma_wall_strike": wall_strike,
        "gamma_wall_exposure": round(wall_gamma, 2),
        "wall_type": wall_type,
        "top_strikes": [{"strike": s, "gamma": round(g, 2)} for s, g in top_5],
        "call_oi_strikes": {str(int(k)): round(v, 4) for k, v in sorted(call_oi_by_strike.items()) if v > 0.1},
        "put_oi_strikes": {str(int(k)): round(v, 4) for k, v in sorted(put_oi_by_strike.items()) if v > 0.1},
        "total_strikes_with_oi": len(gamma_by_strike),
        "source": "okx",
    }


def okx_skew(inst_family: str) -> dict:
    """Compute 25-delta risk reversal (skew) from OKX option chain.

    Skew = 25d Put IV - 25d Call IV.
    Positive skew → puts are more expensive → market is paying for downside protection (bearish).
    Negative skew → calls are more expensive → market is bidding for upside (bullish).

    Uses the nearest weekly expiry for most liquid data.
    """
    if not inst_family:
        return {"status": "unavailable", "reason": "No options market"}

    # Use opt-summary which has full Greeks (deltaBS, markVol, bidVol, askVol)
    summary_data = fetch(f"{OKX_BASE}/public/opt-summary?instFamily={inst_family}", timeout=15)
    summary_items = okx_data(summary_data) if not is_error(summary_data) else []

    if not summary_items:
        return {"status": "unavailable"}

    # Group by expiry
    from collections import defaultdict
    by_expiry = defaultdict(list)
    for t in summary_items:
        inst_id = t.get("instId", "")
        parts = inst_id.split("-")
        if len(parts) != 5:
            continue
        exp = parts[2]
        strike = safe_float(parts[3])
        cp = parts[4]
        delta = safe_float(t.get("deltaBS"))  # Black-Scholes delta
        mark_vol = safe_float(t.get("markVol"))  # Mark IV
        ask_vol = safe_float(t.get("askVol"))
        bid_vol = safe_float(t.get("bidVol"))
        mid_vol = (ask_vol + bid_vol) / 2 if ask_vol > 0 and bid_vol > 0 else mark_vol

        if mid_vol <= 0:
            continue

        by_expiry[exp].append({
            "strike": strike, "cp": cp, "delta": delta,
            "iv": mid_vol, "mark_vol": mark_vol,
        })

    if not by_expiry:
        return {"status": "unavailable"}

    # Use nearest expiry with enough data
    sorted_expiries = sorted(by_expiry.keys())
    target_exp = None
    for exp in sorted_expiries:
        if len(by_expiry[exp]) >= 6:  # need at least a few strikes
            target_exp = exp
            break

    if not target_exp:
        return {"status": "unavailable"}

    options = by_expiry[target_exp]

    # Find 25-delta put and 25-delta call
    # 25d call: delta ≈ +0.25, 25d put: delta ≈ -0.25
    calls = [o for o in options if o["cp"] == "C"]
    puts = [o for o in options if o["cp"] == "P"]

    # Find closest to 25-delta
    call_25d = min(calls, key=lambda o: abs(abs(o["delta"]) - 0.25), default=None) if calls else None
    put_25d = min(puts, key=lambda o: abs(abs(o["delta"]) - 0.25), default=None) if puts else None

    # Find ATM (closest to 50-delta)
    atm_call = min(calls, key=lambda o: abs(abs(o["delta"]) - 0.5), default=None) if calls else None

    if not call_25d or not put_25d:
        return {"status": "unavailable"}

    skew_25d = put_25d["iv"] - call_25d["iv"]  # positive = bearish
    atm_iv = atm_call["iv"] if atm_call else (call_25d["iv"] + put_25d["iv"]) / 2

    # Butterfly: (25d put IV + 25d call IV) / 2 - ATM IV
    # Measures tail risk pricing
    butterfly = ((put_25d["iv"] + call_25d["iv"]) / 2) - atm_iv if atm_call else None

    # Interpretation
    if skew_25d > 0.05:
        skew_signal = "heavily bearish — puts very expensive"
    elif skew_25d > 0.02:
        skew_signal = "bearish — downside protection bid"
    elif skew_25d > -0.02:
        skew_signal = "neutral"
    elif skew_25d > -0.05:
        skew_signal = "bullish — calls bid over puts"
    else:
        skew_signal = "heavily bullish — call skew dominant"

    return {
        "status": "available",
        "expiry": target_exp,
        "skew_25d": round(skew_25d * 100, 2),  # as percentage points
        "put_25d_iv": round(put_25d["iv"] * 100, 2),
        "call_25d_iv": round(call_25d["iv"] * 100, 2),
        "put_25d_strike": put_25d["strike"],
        "call_25d_strike": call_25d["strike"],
        "atm_iv": round(atm_iv * 100, 2),
        "butterfly": round(butterfly * 100, 2) if butterfly is not None else None,
        "signal": skew_signal,
        "source": "okx",
    }


# ═══════════════════════════════════════════════════════════════════
# BINANCE APIs (FALLBACK)
# ═══════════════════════════════════════════════════════════════════

def binance_funding(symbol: str) -> dict:
    """Funding rate fallback from Binance."""
    data = fetch(f"{BINANCE_FAPI}/fapi/v1/fundingRate?symbol={symbol}&limit=1")
    if is_error(data) or not data:
        return {"status": "unavailable"}
    item = data[0] if isinstance(data, list) else data
    rate = safe_float(item.get("fundingRate"))
    return {
        "status": "available",
        "rate": rate,
        "rate_pct": round(rate * 100, 6),
        "rate_annualized_pct": round(rate * 3 * 365 * 100, 2),
        "source": "binance_fallback",
    }


def binance_oi(symbol: str) -> dict:
    """Open interest fallback from Binance."""
    data = fetch(f"{BINANCE_FAPI}/fapi/v1/openInterest?symbol={symbol}")
    if is_error(data):
        return {"status": "unavailable"}
    return {
        "status": "available",
        "oi": safe_float(data.get("openInterest")),
        "source": "binance_fallback",
    }


def binance_long_short(symbol: str) -> dict:
    """Long/short + taker ratio from Binance (fallback)."""
    has_data = False
    result = {"source": "binance_fallback"}

    top = fetch(f"{BINANCE_FAPI}/futures/data/topLongShortPositionRatio?symbol={symbol}&period=1h&limit=1")
    if not is_error(top) and top:
        item = top[0] if isinstance(top, list) else top
        result["top_long_ratio"] = safe_float(item.get("longAccount"))
        result["top_short_ratio"] = safe_float(item.get("shortAccount"))
        has_data = True

    glob = fetch(f"{BINANCE_FAPI}/futures/data/globalLongShortAccountRatio?symbol={symbol}&period=1h&limit=1")
    if not is_error(glob) and glob:
        item = glob[0] if isinstance(glob, list) else glob
        result["global_long_ratio"] = safe_float(item.get("longAccount"))
        result["global_short_ratio"] = safe_float(item.get("shortAccount"))
        has_data = True

    taker = fetch(f"{BINANCE_FAPI}/futures/data/takerlongshortRatio?symbol={symbol}&period=1h&limit=1")
    if not is_error(taker) and taker:
        item = taker[0] if isinstance(taker, list) else taker
        result["taker_buy_sell_ratio"] = safe_float(item.get("buySellRatio"))
        has_data = True

    result["status"] = "available" if has_data else "unavailable"
    return result


def binance_liquidation_proxy(symbol: str) -> dict:
    """Estimate liquidation pressure using Binance long/short ratio history.

    Since Binance deprecated /fapi/v1/allForceOrders, we use the L/S ratio
    trend as a proxy for liquidation pressure: sharp drops in the dominant
    side suggest forced position closures.
    """
    data = fetch(
        f"{BINANCE_FAPI}/futures/data/globalLongShortAccountRatio?symbol={symbol}&period=1h&limit=12",
        timeout=8,
    )
    if is_error(data) or not isinstance(data, list) or len(data) < 2:
        return {"status": "unavailable"}

    ratios = [safe_float(d.get("longShortRatio")) for d in data if isinstance(d, dict)]
    if len(ratios) < 2:
        return {"status": "unavailable"}

    latest = ratios[0]  # most recent
    avg = sum(ratios) / len(ratios)
    max_r = max(ratios)
    min_r = min(ratios)
    swing = max_r - min_r

    # Detect if ratio swung hard (suggesting liquidation cascade)
    if swing > 0.15:
        pressure = "high — significant position unwind detected"
    elif swing > 0.08:
        pressure = "moderate — some forced closures likely"
    else:
        pressure = "low — orderly market"

    bias = ("longs under pressure" if latest < avg
            else "shorts under pressure" if latest > avg
            else "balanced")

    return {
        "status": "available",
        "latest_ls_ratio": round(latest, 4),
        "avg_ls_ratio_12h": round(avg, 4),
        "ratio_swing_12h": round(swing, 4),
        "pressure": pressure,
        "bias": bias,
        "long_pct": round(latest / (latest + 1) * 100, 1) if latest > 0 else 50.0,
        "data_points": len(ratios),
        "source": "binance",
    }


# ═══════════════════════════════════════════════════════════════════
# MACRO / SENTIMENT (token-independent)
# ═══════════════════════════════════════════════════════════════════

def fear_greed() -> dict:
    """Fear & Greed Index."""
    data = fetch("https://api.alternative.me/fng/?limit=7")
    if is_error(data):
        return {"status": "unavailable"}
    items = data.get("data", [])
    if not items:
        return {"status": "unavailable"}
    current = items[0]
    value = int(current.get("value", 0))
    classification = current.get("value_classification", "Unknown")

    # 7-day trend
    values = [int(i.get("value", 0)) for i in items]
    if len(values) >= 2:
        recent = sum(values[:3]) / min(3, len(values))
        older = sum(values[3:]) / max(len(values) - 3, 1)
        trend = "improving" if recent > older + 3 else "deteriorating" if recent < older - 3 else "flat"
    else:
        trend = "unknown"

    return {
        "status": "available",
        "value": value,
        "classification": classification,
        "trend_7d": trend,
        "history_7d": [{"value": int(i.get("value", 0)), "label": i.get("value_classification", ""),
                        "date": i.get("timestamp", "")} for i in items],
        "source": "alternative.me",
    }


def coingecko_global() -> dict:
    """Global market data from CoinGecko."""
    data = fetch("https://api.coingecko.com/api/v3/global", timeout=15)
    if is_error(data):
        return {"status": "unavailable"}
    gd = data.get("data", {})
    if not gd:
        return {"status": "unavailable"}
    return {
        "status": "available",
        "btc_dominance": round(gd.get("market_cap_percentage", {}).get("btc", 0), 2),
        "eth_dominance": round(gd.get("market_cap_percentage", {}).get("eth", 0), 2),
        "total_market_cap_usd": gd.get("total_market_cap", {}).get("usd", 0),
        "total_volume_24h_usd": gd.get("total_volume", {}).get("usd", 0),
        "market_cap_change_24h_pct": round(gd.get("market_cap_change_percentage_24h_usd", 0), 2),
        "source": "coingecko",
    }


def stablecoin_data() -> dict:
    """Stablecoin market cap from DefiLlama."""
    data = fetch("https://stablecoins.llama.fi/stablecoins?includePrices=true", timeout=15)
    if is_error(data):
        return {"status": "unavailable"}
    coins = data.get("peggedAssets", [])
    total_mcap = 0
    top_stables = []
    for c in coins[:5]:
        chains = c.get("chainCirculating", {})
        mcap = sum(v.get("current", {}).get("peggedUSD", 0) for v in chains.values()) if chains else 0
        if mcap == 0:
            mcap = c.get("circulating", {}).get("peggedUSD", 0)
        total_mcap += mcap
        top_stables.append({"name": c.get("name", ""), "symbol": c.get("symbol", ""), "mcap": round(mcap)})
    return {
        "status": "available",
        "total_stablecoin_mcap": round(total_mcap),
        "top_stablecoins": top_stables,
        "source": "defillama",
    }


# ═══════════════════════════════════════════════════════════════════
# PER-TOKEN AGGREGATOR (OKX-first with fallback)
# ═══════════════════════════════════════════════════════════════════

def analyze_token(symbol: str) -> dict:
    """Fetch all available indicators for a token. OKX primary, Binance fallback."""
    token = TOKEN_MAP.get(symbol.upper())
    if not token:
        return {"symbol": symbol, "error": f"Unknown token. Supported: {', '.join(sorted(TOKEN_MAP.keys()))}"}

    result = {
        "symbol": symbol.upper(),
        "tier": token["tier"],
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "derivatives": {},
        "market_structure": {},
    }

    swap_id = token["okx_swap"]
    spot_id = token["okx_spot"]
    bn_sym = token["binance"]
    ccy = symbol.upper()

    # ── Ticker (OKX spot) ──
    result["market_structure"]["ticker_24h"] = okx_ticker(spot_id)

    # ── Realized Volatility (from hourly candles) ──
    closes = okx_candle_history(spot_id, bar="1H", limit=24)
    result["market_structure"]["realized_vol"] = compute_realized_volatility(closes)

    # ── Funding (OKX first → Binance fallback) ──
    funding = okx_funding(swap_id)
    if funding["status"] == "unavailable":
        funding = binance_funding(bn_sym)
    result["derivatives"]["funding"] = funding

    # ── Funding History (OKX, for trend analysis) ──
    result["derivatives"]["funding_history"] = okx_funding_history(swap_id, limit=6)

    # ── Open Interest (OKX first → Binance fallback) ──
    oi = okx_open_interest(swap_id)
    if oi["status"] == "unavailable":
        oi = binance_oi(bn_sym)
    result["derivatives"]["open_interest"] = oi

    # ── OI History / Delta ──
    result["derivatives"]["oi_history"] = okx_oi_history(ccy)

    # ── Cross-exchange OI comparison ──
    bn_oi = binance_oi(bn_sym)
    if oi.get("source") == "okx" and bn_oi["status"] == "available":
        result["derivatives"]["cross_exchange_oi"] = {
            "status": "available",
            "okx_oi": oi.get("oi", 0),
            "binance_oi": bn_oi.get("oi", 0),
            "source": "okx+binance",
        }

    # ── MVRV + Realized Price (BTC/ETH only — CoinMetrics free) ──
    spot_price = (result["market_structure"].get("ticker_24h") or {}).get("price", 0)
    if ccy in ("BTC", "ETH"):
        result["on_chain"] = {"mvrv": coinmetrics_mvrv(ccy, spot_price)}

    # ── Options (Tier 1 only — OKX) ──
    result["derivatives"]["options"] = okx_options_summary(token["okx_family"])

    # ── Gamma Wall (from full option chain) ──
    if token["okx_family"]:
        result["derivatives"]["gamma_wall"] = okx_gamma_wall(token["okx_family"], spot_price)

    # ── Options Skew (25-delta risk reversal) ──
    if token["okx_family"]:
        result["derivatives"]["skew"] = okx_skew(token["okx_family"])

    # ── Futures Basis (OKX swap vs spot) ──
    result["derivatives"]["basis"] = okx_futures_basis(swap_id, spot_id)

    # ── Long/Short Ratio (OKX first → Binance fallback) ──
    ls = okx_long_short(swap_id)
    if ls["status"] == "unavailable":
        ls = binance_long_short(bn_sym)
    result["market_structure"]["long_short"] = ls

    # ── Taker Buy/Sell (OKX, using ccy) ──
    result["market_structure"]["taker_volume"] = okx_taker_volume(ccy)

    # ── Liquidations (Binance) ──
    result["market_structure"]["liquidations"] = binance_liquidation_proxy(bn_sym)

    # ── Cross-exchange funding comparison ──
    if funding.get("source") == "okx":
        bn_funding = binance_funding(bn_sym)
        if bn_funding["status"] == "available":
            result["derivatives"]["binance_funding"] = bn_funding
            # Funding divergence signal
            okx_rate = funding.get("rate", 0)
            bn_rate = bn_funding.get("rate", 0)
            divergence = abs(okx_rate - bn_rate)
            result["derivatives"]["funding_divergence"] = {
                "status": "available",
                "okx_rate_pct": round(okx_rate * 100, 6),
                "binance_rate_pct": round(bn_rate * 100, 6),
                "divergence_pct": round(divergence * 100, 6),
                "signal": (
                    "high divergence — potential arb opportunity" if divergence > 0.0003
                    else "moderate divergence" if divergence > 0.0001
                    else "aligned"
                ),
            }

    time.sleep(0.15)  # rate limit courtesy
    return result


# ═══════════════════════════════════════════════════════════════════
# MAIN
# ═══════════════════════════════════════════════════════════════════

def main():
    tokens = sys.argv[1:] if len(sys.argv) > 1 else ["BTC"]
    if tokens == ["--all"]:
        tokens = sorted(TOKEN_MAP.keys())

    output = {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "version": "2.0",
        "data_priority": "OKX primary, Binance fallback",
        "tokens": {},
        "macro": {},
    }

    # Macro (token-independent)
    print("Fetching macro data...", file=sys.stderr)
    output["macro"]["fear_greed"] = fear_greed()
    output["macro"]["global"] = coingecko_global()
    output["macro"]["stablecoins"] = stablecoin_data()

    # Per-token
    for t in tokens:
        t = t.upper()
        print(f"Fetching {t} (OKX primary, Binance fallback)...", file=sys.stderr)
        output["tokens"][t] = analyze_token(t)
        time.sleep(0.2)

    # Summary
    available = 0
    unavailable = 0
    for t_data in output["tokens"].values():
        for section in ["derivatives", "market_structure", "on_chain"]:
            for k, v in t_data.get(section, {}).items():
                if isinstance(v, dict):
                    if v.get("status") == "available":
                        available += 1
                    elif v.get("status") == "unavailable":
                        unavailable += 1
    for k, v in output["macro"].items():
        if isinstance(v, dict):
            if v.get("status") == "available":
                available += 1
            elif v.get("status") == "unavailable":
                unavailable += 1

    output["_summary"] = {
        "indicators_available": available,
        "indicators_unavailable": unavailable,
        "coverage_pct": round(available / max(available + unavailable, 1) * 100, 1),
    }

    print(json.dumps(output, indent=2))


if __name__ == "__main__":
    main()
