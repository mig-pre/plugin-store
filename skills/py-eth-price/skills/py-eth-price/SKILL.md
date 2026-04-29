---
name: py-eth-price
description: "Query real-time ETH price via OKX API"
version: "1.0.0"
author: "yz06276"
tags:
  - ethereum
  - price
---


## Pre-flight Dependencies (auto-injected by Plugin Store CI)

> Run once per session before first use. These checks ensure required tools are installed.

### Version check (auto-injected)

> **AI Agent Instruction**: Run this version check before executing any commands from this skill. If an update is applied, re-read this SKILL.md before proceeding — the instructions may have changed.

```bash
# Check for skill updates (1-hour cache)
UPDATE_CACHE="$HOME/.plugin-store/update-cache/py-eth-price"
CACHE_MAX=3600
LOCAL_VER="1.0.0"
DO_CHECK=true

if [ -f "$UPDATE_CACHE" ]; then
  CACHE_MOD=$(stat -f %m "$UPDATE_CACHE" 2>/dev/null || stat -c %Y "$UPDATE_CACHE" 2>/dev/null || echo 0)
  NOW=$(date +%s)
  AGE=$(( NOW - CACHE_MOD ))
  [ "$AGE" -lt "$CACHE_MAX" ] && DO_CHECK=false
fi

if [ "$DO_CHECK" = true ]; then
  REMOTE_VER=$(curl -sf --max-time 3 "https://raw.githubusercontent.com/mig-pre/plugin-store/main/skills/py-eth-price/plugin.yaml" | grep '^version' | head -1 | tr -d '"' | awk '{print $2}')
  if [ -n "$REMOTE_VER" ]; then
    mkdir -p "$HOME/.plugin-store/update-cache"
    echo "$REMOTE_VER" > "$UPDATE_CACHE"
  fi
fi

REMOTE_VER=$(cat "$UPDATE_CACHE" 2>/dev/null || echo "$LOCAL_VER")
if [ "$REMOTE_VER" != "$LOCAL_VER" ]; then
  echo "Update available: py-eth-price v$LOCAL_VER -> v$REMOTE_VER. Updating..."
  npx skills add mig-pre/plugin-store --skill py-eth-price --yes --global 2>/dev/null || true
  echo "Updated py-eth-price to v$REMOTE_VER. Please re-read this SKILL.md."
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

---


# py-eth-price

## Overview

A Python script that queries the current ETH price from the OKX public API.

## Pre-flight Checks

Before using this skill, ensure:

1. Python 3 is installed
2. The `onchainos` CLI is installed and configured

## Commands

### Get ETH Price

```bash
python3 src/main.py
```

**When to use**: When the user asks for the current price of ETH.
**Output**: JSON with price, 24h change, and volume.

### Get ETH Price via onchainos

```bash
onchainos market price --token ETH --chain ethereum
```

**When to use**: Alternative method using onchainos directly.

## Error Handling

| Error | Cause | Resolution |
|-------|-------|------------|
| "Connection error" | No internet | Check connectivity and retry |
| "Rate limited" | Too many requests | Wait 10 seconds and retry |
