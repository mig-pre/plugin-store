---
name: go-price-demo
description: "Query ETH/BTC prices via OKX public API — a Go demo plugin for CI testing"
license: MIT
metadata:
  author: yz06276
  version: "0.1.0"
---


## Pre-flight Dependencies (auto-injected by Plugin Store CI)

> Run once per session before first use. These checks ensure required tools are installed.

### Version check (auto-injected)

> **AI Agent Instruction**: Run this version check before executing any commands from this skill. If an update is applied, re-read this SKILL.md before proceeding — the instructions may have changed.

```bash
# Check for skill updates (1-hour cache)
UPDATE_CACHE="$HOME/.plugin-store/update-cache/go-price-demo"
CACHE_MAX=3600
LOCAL_VER="0.1.0"
DO_CHECK=true

if [ -f "$UPDATE_CACHE" ]; then
  CACHE_MOD=$(stat -f %m "$UPDATE_CACHE" 2>/dev/null || stat -c %Y "$UPDATE_CACHE" 2>/dev/null || echo 0)
  NOW=$(date +%s)
  AGE=$(( NOW - CACHE_MOD ))
  [ "$AGE" -lt "$CACHE_MAX" ] && DO_CHECK=false
fi

if [ "$DO_CHECK" = true ]; then
  REMOTE_VER=$(curl -sf --max-time 3 "https://raw.githubusercontent.com/mig-pre/plugin-store/main/skills/go-price-demo/plugin.yaml" | grep '^version' | head -1 | tr -d '"' | awk '{print $2}')
  if [ -n "$REMOTE_VER" ]; then
    mkdir -p "$HOME/.plugin-store/update-cache"
    echo "$REMOTE_VER" > "$UPDATE_CACHE"
  fi
fi

REMOTE_VER=$(cat "$UPDATE_CACHE" 2>/dev/null || echo "$LOCAL_VER")
if [ "$REMOTE_VER" != "$LOCAL_VER" ]; then
  echo "Update available: go-price-demo v$LOCAL_VER -> v$REMOTE_VER. Updating..."
  npx skills add mig-pre/plugin-store --skill go-price-demo --yes --global 2>/dev/null || true
  echo "Updated go-price-demo to v$REMOTE_VER. Please re-read this SKILL.md."
fi
```

### Install go-price-demo binary + launcher (auto-injected)

```bash
# Install shared infrastructure (launcher + update checker, only once)
LAUNCHER="$HOME/.plugin-store/launcher.sh"
CHECKER="$HOME/.plugin-store/update-checker.py"
if [ ! -f "$LAUNCHER" ]; then
  mkdir -p "$HOME/.plugin-store"
  curl -fsSL "https://raw.githubusercontent.com/mig-pre/plugin-store/main/scripts/launcher.sh" -o "$LAUNCHER" 2>/dev/null || true
  chmod +x "$LAUNCHER"
fi
if [ ! -f "$CHECKER" ]; then
  curl -fsSL "https://raw.githubusercontent.com/mig-pre/plugin-store/main/scripts/update-checker.py" -o "$CHECKER" 2>/dev/null || true
fi

# Clean up old installation
rm -f "$HOME/.local/bin/go-price-demo" "$HOME/.local/bin/.go-price-demo-core" 2>/dev/null

# Download binary
OS=$(uname -s | tr A-Z a-z)
ARCH=$(uname -m)
EXT=""
case "${OS}_${ARCH}" in
  darwin_arm64)  TARGET="aarch64-apple-darwin" ;;
  darwin_x86_64) TARGET="x86_64-apple-darwin" ;;
  linux_x86_64)  TARGET="x86_64-unknown-linux-musl" ;;
  linux_i686)    TARGET="i686-unknown-linux-musl" ;;
  linux_aarch64) TARGET="aarch64-unknown-linux-musl" ;;
  linux_armv7l)  TARGET="armv7-unknown-linux-musleabihf" ;;
  mingw*_x86_64|msys*_x86_64|cygwin*_x86_64)   TARGET="x86_64-pc-windows-msvc"; EXT=".exe" ;;
  mingw*_i686|msys*_i686|cygwin*_i686)           TARGET="i686-pc-windows-msvc"; EXT=".exe" ;;
  mingw*_aarch64|msys*_aarch64|cygwin*_aarch64)  TARGET="aarch64-pc-windows-msvc"; EXT=".exe" ;;
esac
mkdir -p ~/.local/bin
curl -fsSL "https://github.com/mig-pre/plugin-store/releases/download/plugins/go-price-demo@0.1.0/go-price-demo-${TARGET}${EXT}" -o ~/.local/bin/.go-price-demo-core${EXT}
chmod +x ~/.local/bin/.go-price-demo-core${EXT}

# Symlink CLI name to universal launcher
ln -sf "$LAUNCHER" ~/.local/bin/go-price-demo

# Register version
mkdir -p "$HOME/.plugin-store/managed"
echo "0.1.0" > "$HOME/.plugin-store/managed/go-price-demo"
```

### Report install (auto-injected, runs once)

```bash
REPORT_FLAG="$HOME/.plugin-store/reported/go-price-demo"
if [ ! -f "$REPORT_FLAG" ]; then
  mkdir -p "$HOME/.plugin-store/reported"
  # Device fingerprint → SHA256 → 32-char device ID
  DEV_RAW="$(hostname):$(uname -s):$(uname -m):$HOME"
  DEV_ID=$(echo -n "$DEV_RAW" | shasum -a 256 | head -c 32)
  # HMAC signature (obfuscated key, same as CLI binary)
  _K=$(echo 'OE9nNWFRUFdfSVJkektrMExOV2RNeTIzV2JibXo3ZWNTbExJUDFIWnVoZw==' | base64 -d 2>/dev/null || echo 'OE9nNWFRUFdfSVJkektrMExOV2RNeTIzV2JibXo3ZWNTbExJUDFIWnVoZw==' | openssl base64 -d)
  HMAC_SIG=$(echo -n "${_K}${DEV_ID}" | shasum -a 256 | head -c 8)
  DIV_ID="${DEV_ID}${HMAC_SIG}"
  unset _K
  # Report to Vercel stats
  curl -s -X POST "https://plugin-store-dun.vercel.app/install" \
    -H "Content-Type: application/json" \
    -d '{"name":"go-price-demo","version":"0.1.0"}' >/dev/null 2>&1 || true
  # Report to OKX API (with HMAC-signed device token)
  curl -s -X POST "https://www.okx.com/priapi/v1/wallet/plugins/download/report" \
    -H "Content-Type: application/json" \
    -d '{"pluginName":"go-price-demo","divId":"'"$DIV_ID"'"}' >/dev/null 2>&1 || true
  touch "$REPORT_FLAG"
fi
```

---


# Go Price Demo

A minimal Go demo plugin that queries cryptocurrency prices from the OKX public API.

## Commands

### get-price — Get Token Price

```bash
go-price-demo get-price --token <SYMBOL>
```

**Parameters:**
| Parameter | Default | Description |
|-----------|---------|-------------|
| `--token` | `ETH` | Token symbol (e.g. ETH, BTC, SOL) |

**Example:**
```bash
go-price-demo get-price --token ETH
go-price-demo get-price --token BTC
```

**Output (JSON):**
```json
{
  "ok": true,
  "token": "ETH",
  "price_usd": "2327.01",
  "open_24h": "2310.50",
  "high_24h": "2415.00",
  "low_24h": "2295.30",
  "volume_24h": "441282371.62",
  "change_24h_pct": "0.72",
  "source": "okx-public-api"
}
```

## Data Sources

- **OKX Public API**: `GET https://www.okx.com/api/v5/market/ticker?instId={TOKEN}-USDT`

## Safety

- Read-only plugin — no transactions, no wallet access
- No API keys required (uses public endpoints only)
- No sensitive data stored or transmitted
