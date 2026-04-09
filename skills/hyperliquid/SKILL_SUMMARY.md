
# hyperliquid -- Skill Summary

## Overview
This skill enables trading perpetual futures on Hyperliquid, a high-performance on-chain perpetuals exchange built on its own L1 blockchain. It provides comprehensive trading functionality including position management, order placement with stop-loss/take-profit brackets, market data retrieval, and USDC deposits from Arbitrum. All operations use USDC as the margin token and settle on Hyperliquid L1 with CEX-like speed but full on-chain transparency.

## Usage
Install the hyperliquid binary and ensure onchainos CLI is configured with your wallet. For write operations, first run commands without `--confirm` to preview, then add `--confirm` to sign and execute via EIP-712 signatures.

## Commands
| Command | Description |
|---------|-------------|
| `hyperliquid positions` | Check open perpetual positions and account summary |
| `hyperliquid prices` | Get current mid prices for all or specific markets |
| `hyperliquid order` | Place market/limit orders with optional TP/SL brackets |
| `hyperliquid close` | Market-close an open position |
| `hyperliquid tpsl` | Set stop-loss/take-profit on existing positions |
| `hyperliquid cancel` | Cancel open orders by order ID |
| `hyperliquid deposit` | Deposit USDC from Arbitrum to Hyperliquid |

## Triggers
Activate when users mention trading on Hyperliquid, checking Hyperliquid positions, placing perp orders, or managing stop-loss/take-profit levels. Also triggers for Hyperliquid-specific terms like "HL order", "HYPE perps", or phrases about opening/closing positions on the platform.
