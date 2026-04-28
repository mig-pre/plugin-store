---
name: uniswap-v4-security-foundations
description: "Security-first guide for building Uniswap v4 hooks covering vulnerabilities, audit requirements, and best practices"
version: "1.1.0"
author: "Uniswap Labs"
tags:
  - uniswap
  - defi
---


## Pre-flight Dependencies (auto-injected by Plugin Store CI)

> Run once per session before first use. These checks ensure required tools are installed.

### Version check (auto-injected)

> **AI Agent Instruction**: Run this version check before executing any commands from this skill. If an update is applied, re-read this SKILL.md before proceeding — the instructions may have changed.

```bash
# Check for skill updates (1-hour cache)
UPDATE_CACHE="$HOME/.plugin-store/update-cache/uniswap-v4-security-foundations"
CACHE_MAX=3600
LOCAL_VER="1.1.0"
DO_CHECK=true

if [ -f "$UPDATE_CACHE" ]; then
  CACHE_MOD=$(stat -f %m "$UPDATE_CACHE" 2>/dev/null || stat -c %Y "$UPDATE_CACHE" 2>/dev/null || echo 0)
  NOW=$(date +%s)
  AGE=$(( NOW - CACHE_MOD ))
  [ "$AGE" -lt "$CACHE_MAX" ] && DO_CHECK=false
fi

if [ "$DO_CHECK" = true ]; then
  REMOTE_VER=$(curl -sf --max-time 3 "https://raw.githubusercontent.com/okx/plugin-store/main/skills/uniswap-v4-security-foundations/plugin.yaml" | grep '^version' | head -1 | tr -d '"' | awk '{print $2}')
  if [ -n "$REMOTE_VER" ]; then
    mkdir -p "$HOME/.plugin-store/update-cache"
    echo "$REMOTE_VER" > "$UPDATE_CACHE"
  fi
fi

REMOTE_VER=$(cat "$UPDATE_CACHE" 2>/dev/null || echo "$LOCAL_VER")
if [ "$REMOTE_VER" != "$LOCAL_VER" ]; then
  echo "Update available: uniswap-v4-security-foundations v$LOCAL_VER -> v$REMOTE_VER. Updating..."
  npx skills add okx/plugin-store --skill uniswap-v4-security-foundations --yes --global 2>/dev/null || true
  echo "Updated uniswap-v4-security-foundations to v$REMOTE_VER. Please re-read this SKILL.md."
fi
```

---


# uniswap-v4-security-foundations

This skill is maintained by Uniswap Labs. Install the full version:

```
npx skills add Uniswap/uniswap-ai
```

Or install just this plugin:

```
claude plugin add @uniswap/uniswap-hooks
```

Source: [uniswap-ai/packages/plugins/uniswap-hooks/skills/v4-security-foundations](https://github.com/uniswap/uniswap-ai/tree/main/packages/plugins/uniswap-hooks/skills/v4-security-foundations)
