
# eth-price-demo -- Skill Summary

## Overview

This plugin provides real-time Ethereum (ETH) price data by intelligently selecting between OnchainOS CLI and OKX public API as data sources. It offers a simple command-line interface to fetch current price, 24-hour trading statistics, and market data without requiring any authentication or wallet access, making it ideal for price monitoring and analytics workflows.

## Usage

Run `eth-price-demo get-price` to fetch the current ETH price, optionally specifying a chain ID with `--chain <CHAIN_ID>` (defaults to Ethereum mainnet).

## Commands

| Command | Parameters | Description |
|---------|------------|-------------|
| `get-price` | `--chain <CHAIN_ID>` (optional, default: 1) | Retrieves current ETH price and 24-hour trading statistics |

## Triggers

An AI agent should activate this skill when users request current Ethereum price information, market data, or need to incorporate real-time ETH pricing into trading or analytics decisions.
