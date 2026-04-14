
# polymarket-plugin -- Skill Summary

## Overview
This skill enables AI agents to interact with Polymarket prediction markets on Polygon blockchain. Users can browse active markets, place trades on real-world event outcomes (elections, sports, crypto price targets, etc.), manage positions, and handle funds. The plugin supports both direct wallet trading and gasless proxy wallet mode, with automatic credential derivation and EIP-712 order signing through the onchainos CLI integration.

## Usage
Install the plugin, connect your wallet via `onchainos wallet login`, verify region access with `check-access`, fund with USDC.e on Polygon, then start trading with `buy` and `sell` commands.

## Commands
| Command | Description |
|---------|-------------|
| `check-access` | Verify region is not restricted |
| `list-markets` | Browse active prediction markets with filtering options |
| `list-5m` | List 5-minute crypto up/down markets |
| `get-market` | Get market details and order book data |
| `get-positions` | View open positions and P&L |
| `balance` | Show POL and USDC.e balances |
| `buy` | Buy YES/NO outcome shares |
| `sell` | Sell outcome shares |
| `cancel` | Cancel an open order |
| `redeem` | Redeem winning tokens after market resolution |
| `setup-proxy` | Deploy proxy wallet for gasless trading |
| `deposit` | Transfer USDC.e from EOA to proxy wallet |
| `switch-mode` | Switch between EOA and proxy trading modes |

## Triggers
Activate when users want to trade prediction markets, bet on real-world events, check market prices, or use phrases like "buy polymarket shares," "bet on," "prediction market," "5-minute market," or when they express interest in trading outcomes for elections, sports, crypto prices, or trending news events.
