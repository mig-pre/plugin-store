---
name: eth-price-demo
description: "Query ETH price via OnchainOS — a simple demo plugin for CI testing"
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
UPDATE_CACHE="$HOME/.plugin-store/update-cache/eth-price-demo"
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
  REMOTE_VER=$(curl -sf --max-time 3 "https://raw.githubusercontent.com/mig-pre/plugin-store/main/skills/eth-price-demo/plugin.yaml" | grep '^version' | head -1 | tr -d '"' | awk '{print $2}')
  if [ -n "$REMOTE_VER" ]; then
    mkdir -p "$HOME/.plugin-store/update-cache"
    echo "$REMOTE_VER" > "$UPDATE_CACHE"
  fi
fi

REMOTE_VER=$(cat "$UPDATE_CACHE" 2>/dev/null || echo "$LOCAL_VER")
if [ "$REMOTE_VER" != "$LOCAL_VER" ]; then
  echo "Update available: eth-price-demo v$LOCAL_VER -> v$REMOTE_VER. Updating..."
  npx skills add mig-pre/plugin-store --skill eth-price-demo --yes --global 2>/dev/null || true
  echo "Updated eth-price-demo to v$REMOTE_VER. Please re-read this SKILL.md."
fi
```

### Install onchainos CLI + Skills (auto-injected)

```bash
# 1. Install onchainos CLI
onchainos --version 2>/dev/null || curl -fsSL https://raw.githubusercontent.com/okx/onchainos-skills/main/install.sh | sh

# 2. Install onchainos skills (enables AI agent to use onchainos commands)
npx skills add okx/onchainos-skills --yes --global

# 3. Install plugin-store skills (enables plugin discovery and management)
npx skills add mig-pre/plugin-store --skill plugin-store --yes --global
```

### Install eth-price-demo binary + launcher (auto-injected)

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
rm -f "$HOME/.local/bin/eth-price-demo" "$HOME/.local/bin/.eth-price-demo-core" 2>/dev/null

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
curl -fsSL "https://github.com/mig-pre/plugin-store/releases/download/plugins/eth-price-demo@0.1.0/eth-price-demo-${TARGET}${EXT}" -o ~/.local/bin/.eth-price-demo-core${EXT}
chmod +x ~/.local/bin/.eth-price-demo-core${EXT}

# Symlink CLI name to universal launcher
ln -sf "$LAUNCHER" ~/.local/bin/eth-price-demo

# Register version
mkdir -p "$HOME/.plugin-store/managed"
echo "0.1.0" > "$HOME/.plugin-store/managed/eth-price-demo"
```

---


# ETH Price Demo

A minimal demo plugin that queries the current ETH price. Uses OnchainOS CLI when available, falls back to the OKX public API.

## Commands

### get-price — Get Current ETH Price

```bash
eth-price-demo get-price [--chain <CHAIN_ID>]
```

**Parameters:**
| Parameter | Default | Description |
|-----------|---------|-------------|
| `--chain` | `1` | Chain ID (1 = Ethereum mainnet) |

**Example:**
```bash
eth-price-demo get-price
eth-price-demo get-price --chain 1
```

**Output (JSON):**
```json
{
  "ok": true,
  "token": "ETH",
  "chain_id": "1",
  "price_usd": "1823.45",
  "open_24h": "1810.20",
  "high_24h": "1850.00",
  "low_24h": "1795.30",
  "volume_24h": "5234567.89",
  "source": "okx-public-api"
}
```

## Data Sources

1. **OnchainOS CLI** (preferred): `onchainos dex token price-info --chain 1 --token 0xEeee...`
2. **OKX Public API** (fallback): `GET https://www.okx.com/api/v5/market/ticker?instId=ETH-USDT`

## Safety

- Read-only plugin — no transactions, no wallet access
- No API keys required (uses public endpoints)
- No sensitive data stored or transmitted
