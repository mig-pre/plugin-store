"""
Market Structure Analyzer — Configuration
Read-only analytics skill. No trading, no wallet access.
"""

# ── Output ─────────────────────────────────────────────────────────────
OUTPUT_FORMAT = "json"               # "json" / "text"
DEFAULT_TOKENS = ["BTC"]             # Default tokens when none specified

# ── Data Sources ───────────────────────────────────────────────────────
OKX_BASE = "https://www.okx.com/api/v5"
BINANCE_BASE = "https://fapi.binance.com"
COINMETRICS_BASE = "https://community-api.coinmetrics.io/v4"
COINGECKO_BASE = "https://api.coingecko.com/api/v3"
ALTERNATIVE_BASE = "https://api.alternative.me"
DEFILLAMA_BASE = "https://stablecoins.llama.fi"

# ── Rate Limits ────────────────────────────────────────────────────────
REQUEST_TIMEOUT = 10                 # seconds per HTTP request
MAX_RETRIES = 2                      # retry on transient failures

# ── Risk Disclaimer ───────────────────────────────────────────────────
# This skill is READ-ONLY analytics. It does NOT execute trades,
# access wallets, or manage funds. All data is from public APIs.
