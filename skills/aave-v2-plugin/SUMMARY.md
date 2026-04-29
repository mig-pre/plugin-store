## Overview

Aave V2 is the original Aave lending protocol on Ethereum, Polygon, and Avalanche. Supply assets to earn interest, borrow with either stable or variable rates, manage Health Factor, claim stkAAVE / WMATIC / WAVAX incentives. V2 supply and borrow are NOT paused (unlike Compound V2). For new positions, prefer aave-v3-plugin (better gas, isolation mode, eMode); use this V2 plugin for legacy V2 positions, V2-integrated protocols (Maker DSS, Yearn V2 strategies), or stable-rate borrows (V3 removed stable mode).

## Prerequisites
- onchainos CLI installed and logged in
- Native gas on the target chain (mainnet: >=0.005 ETH; Polygon: >=0.1 MATIC; Avalanche: >=0.05 AVAX)
- For exit flows: existing Aave V2 supply or borrow position. Run quickstart to detect.
- For new supply/borrow: a token in your wallet OR existing collateral on V2 main account.

## Quick Start
1. Check your current state and get a guided next step on a specific chain: `aave-v2-plugin quickstart` (defaults to Ethereum; pass `--chain POLYGON` or `--chain AVAX` for other deployments)
2. If you see `status: rpc_degraded` - public RPC failed; wait a minute and retry: `aave-v2-plugin quickstart`
3. If you see `status: insufficient_gas` - top up native gas on the target chain: `aave-v2-plugin markets --chain ETH`
4. If you see `status: ready_to_supply` - copy the recommended next_command to start earning interest: `aave-v2-plugin supply --token USDC --amount 100 --confirm`
5. If you see `status: has_supply_can_borrow` - your supply forms collateral; borrow against it (variable rate by default): `aave-v2-plugin borrow --token USDT --amount 50 --rate-mode 2 --confirm`
6. If you see `status: has_active_borrow` - close debt cleanly (uint256.max sentinel, dust-free): `aave-v2-plugin repay --token USDT --all --rate-mode 2 --confirm`
7. If you see `status: unhealthy_position` - Health Factor below safe margin; immediately repay debt or add collateral. Run `aave-v2-plugin positions` for full breakdown.
8. To exit a supply position back to wallet: `aave-v2-plugin withdraw --token USDC --amount all --confirm`
