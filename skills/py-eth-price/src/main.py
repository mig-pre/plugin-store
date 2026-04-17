#!/usr/bin/env python3
"""Query ETH price from OKX API."""
import json
import sys
import time
import urllib.request

def main():
    url = "https://www.okx.com/api/v5/market/ticker?instId=ETH-USDT"
    try:
        with urllib.request.urlopen(url, timeout=10) as resp:
            data = json.loads(resp.read())
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)

    if not data.get("data"):
        print("Error: No data", file=sys.stderr)
        sys.exit(1)

    t = data["data"][0]
    price = float(t["last"])
    open_24h = float(t["open24h"])
    change = ((price - open_24h) / open_24h) * 100

    result = {
        "token": "ETH",
        "price_usd": f"{price:.2f}",
        "change_24h": f"{change:+.2f}%",
        "volume_24h": t["vol24h"],
        "timestamp": int(time.time()),
    }
    print(json.dumps(result, indent=2))

if __name__ == "__main__":
    main()
