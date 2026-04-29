use clap::Args;
use serde_json::{json, Value};

use crate::config::{ChainInfo, parse_chain, supported_chains_help, SUPPORTED_CHAINS};
use crate::onchainos::resolve_wallet;
use crate::rpc::{
    erc20_balance, erc20_decimals, erc20_symbol, fmt_1e18, fmt_token_amount,
    get_reserves_list, get_user_account_data, incentives_get_unclaimed_rewards,
    lp_get_reserve_data, native_balance, ray_to_apr_pct,
};

const ABOUT: &str = "Aave V2 - the original Aave lending protocol on Ethereum mainnet, Polygon, and Avalanche. Supply assets to earn interest, borrow with stable (V2-only) or variable rates, manage Health Factor, claim stkAAVE/WMATIC/WAVAX rewards. V2 is NOT paused (unlike Compound V2) but V3 is the actively maintained version. Use aave-v3-plugin for new positions; use this plugin for legacy V2 supply/borrow management or V2-integrated protocols.";

/// HF threshold below which we consider a position unhealthy (1.05x).
/// HF is 1e18-scaled; 1.0e18 = liquidation, > 1.0 = safe.
const HF_UNHEALTHY: u128 = 1_050_000_000_000_000_000; // 1.05e18

#[derive(Args)]
pub struct QuickstartArgs {
    /// Chain key or id (ETH / POLYGON / AVAX, or 1 / 137 / 43114). Default: ETH.
    #[arg(long, default_value = "ETH")]
    pub chain: String,
    /// Wallet address to query. Defaults to the connected onchainos wallet on the chosen chain.
    #[arg(long)]
    pub address: Option<String>,
}

struct ReserveScan {
    asset: String,
    symbol: String,
    decimals: u32,
    a_token: String,
    s_debt_token: String,
    v_debt_token: String,
    /// User's wallet balance of the underlying token (NOT yet supplied).
    wallet_balance_raw: u128,
    /// User's current supply (= aToken balance, same units as underlying).
    supply_raw: u128,
    /// User's current variable-rate debt.
    variable_debt_raw: u128,
    /// User's current stable-rate debt.
    stable_debt_raw: u128,
    /// Whether the user's supply in this reserve is enabled as collateral.
    used_as_collateral: bool,
    supply_apr: f64,
    variable_borrow_apr: f64,
    stable_borrow_apr: f64,
}

pub async fn run(args: QuickstartArgs) -> anyhow::Result<()> {
    // 1. Resolve chain
    let chain: &ChainInfo = match parse_chain(&args.chain) {
        Some(c) => c,
        None => {
            println!("{}", super::error_response(
                &format!("Unknown --chain '{}'", args.chain),
                "INVALID_CHAIN",
                &format!("Supported: {}", supported_chains_help()),
            ));
            return Ok(());
        }
    };

    // 2. Resolve wallet for that chain
    let wallet = match &args.address {
        Some(a) => a.clone(),
        None => match resolve_wallet(chain.id) {
            Ok(a) => a,
            Err(e) => {
                println!("{}", super::error_response(
                    &format!("Could not resolve wallet from onchainos for chain {}: {:#}", chain.key, e),
                    "WALLET_NOT_FOUND",
                    "Run `onchainos wallet addresses` to verify login, or pass --address explicitly.",
                ));
                return Ok(());
            }
        },
    };

    eprintln!("Scanning Aave V2 state on {} for {}...", chain.key, &wallet[..std::cmp::min(10, wallet.len())]);

    // 3. Native gas + Account data + Reserves list (parallel)
    let native_fut = native_balance(&wallet, chain.rpc);
    let acct_fut = get_user_account_data(chain.lending_pool, &wallet, chain.rpc);
    let reserves_fut = get_reserves_list(chain.lending_pool, chain.rpc);
    let rewards_fut = async {
        if chain.incentives_controller.is_empty() { Ok(0u128) }
        else { incentives_get_unclaimed_rewards(chain.incentives_controller, &wallet, chain.rpc).await }
    };

    let (native_res, acct_res, reserves_res, rewards_res) = tokio::join!(
        native_fut, acct_fut, reserves_fut, rewards_fut
    );

    let native_bal = native_res.unwrap_or(0);
    let (total_collateral_eth, total_debt_eth, available_borrows_eth, _liq_threshold, _ltv, hf) =
        acct_res.unwrap_or((0, 0, 0, 0, 0, 0));
    let rewards_accrued = rewards_res.unwrap_or(0);

    let reserves: Vec<String> = match reserves_res {
        Ok(r) => r,
        Err(e) => {
            println!("{}", super::error_response(
                &format!("Failed to enumerate reserves on {}: {:#}", chain.key, e),
                "RPC_ERROR",
                "Public RPC may be limited; retry shortly.",
            ));
            return Ok(());
        }
    };

    // 4. Per-reserve parallel scan: tokens, decimals, symbol, user balances, market rates
    let scan_futs: Vec<_> = reserves.iter().map(|asset| {
        let chain = chain.clone();
        let wallet = wallet.clone();
        let asset = asset.clone();
        async move { scan_reserve(&asset, &chain, &wallet).await }
    }).collect();
    let scan_results = futures::future::join_all(scan_futs).await;

    let mut rpc_failures = 0;
    let scans: Vec<ReserveScan> = scan_results.into_iter().filter_map(|r| match r {
        Ok(s) => Some(s),
        Err(_) => { rpc_failures += 1; None }
    }).collect();

    // 5. Aggregate states
    let any_supply = scans.iter().any(|s| s.supply_raw > 0);
    let any_variable_debt = scans.iter().any(|s| s.variable_debt_raw > 0);
    let any_stable_debt = scans.iter().any(|s| s.stable_debt_raw > 0);
    let any_debt = any_variable_debt || any_stable_debt;
    let any_wallet_balance = scans.iter().any(|s| s.wallet_balance_raw > 0);
    let has_gas = native_bal >= chain.gas_floor_wei;
    // HF: 0 means "no debt" or unknown; only flag as unhealthy if HF > 0 (real value) and < threshold
    let unhealthy = hf > 0 && hf < HF_UNHEALTHY && total_debt_eth > 0;

    // 6. Status decision tree (matches SUMMARY.md statuses)
    let total_reserves = reserves.len();
    let (status, next_command, tip): (&str, Option<String>, String) = if rpc_failures >= 3 {
        ("rpc_degraded", None,
         format!("{} of {} reserve scans failed on {}. Public RPC may be limited; retry shortly.",
                 rpc_failures, total_reserves, chain.key))
    } else if unhealthy {
        ("unhealthy_position",
         Some(format!("aave-v2-plugin positions --chain {}", chain.key)),
         format!("Health Factor {:.4} is below safe threshold (1.05). Repay debt or add collateral immediately to avoid liquidation. Run `positions` for the full breakdown.", hf as f64 / 1e18))
    } else if any_debt {
        let debt_scan = scans.iter().find(|s| s.variable_debt_raw > 0 || s.stable_debt_raw > 0).unwrap();
        let (debt_amt, mode) = if debt_scan.variable_debt_raw > 0 {
            (debt_scan.variable_debt_raw, 2)
        } else {
            (debt_scan.stable_debt_raw, 1)
        };
        ("has_active_borrow",
         Some(format!("aave-v2-plugin repay --chain {} --token {} --all --rate-mode {} --confirm",
                      chain.key, debt_scan.symbol, mode)),
         format!("You have {} {} debt ({}-rate). Repay-all uses uint256.max sentinel - settles to exact zero, no dust.",
                 fmt_token_amount(debt_amt, debt_scan.decimals), debt_scan.symbol,
                 if mode == 1 { "stable" } else { "variable" }))
    } else if any_supply {
        let s = scans.iter().filter(|m| m.supply_raw > 0)
            .max_by_key(|m| m.supply_raw).unwrap();
        ("has_supply_can_borrow",
         Some(format!("aave-v2-plugin positions --chain {}", chain.key)),
         format!("You're supplying {} {} on {} (earning {:.2}% APR). Available to borrow: ~{} ETH-equivalent. Use `positions` for the full picture, or `borrow --token X --rate-mode 2 --confirm` to lever up.",
                 fmt_token_amount(s.supply_raw, s.decimals), s.symbol, chain.key,
                 s.supply_apr, fmt_1e18(available_borrows_eth)))
    } else if !has_gas {
        ("insufficient_gas", None,
         format!("Wallet has only {} {} - Aave V2 ops on {} need at least {} {} for gas.",
                 fmt_token_amount(native_bal, 18), chain.native_symbol, chain.key,
                 fmt_token_amount(chain.gas_floor_wei, 18), chain.native_symbol))
    } else if any_wallet_balance {
        let s = scans.iter().filter(|m| m.wallet_balance_raw > 0)
            .max_by_key(|m| m.wallet_balance_raw).unwrap();
        let suggested = sensible_supply_amount(s.wallet_balance_raw, s.decimals);
        ("ready_to_supply",
         Some(format!("aave-v2-plugin supply --chain {} --token {} --amount {} --confirm",
                      chain.key, s.symbol, suggested)),
         format!("You have {} {} in wallet on {} (supply APR {:.2}%). Supply to earn interest.",
                 fmt_token_amount(s.wallet_balance_raw, s.decimals), s.symbol, chain.key, s.supply_apr))
    } else {
        // Has gas but no supportable tokens
        ("ready_to_supply", None,
         format!("You have {} {} gas on {} but no Aave-listed tokens in wallet. Fund with USDC/USDT/DAI/WETH/WBTC/etc, then re-run quickstart. Run `markets` to see all listed reserves.",
                 fmt_token_amount(native_bal, 18), chain.native_symbol, chain.key))
    };

    // 7. Render
    // Show only top 8 reserves with non-zero user balance OR top 8 by supply APR
    let mut display_scans: Vec<&ReserveScan> = scans.iter().collect();
    display_scans.sort_by(|a, b| {
        let a_has = a.wallet_balance_raw > 0 || a.supply_raw > 0
            || a.variable_debt_raw > 0 || a.stable_debt_raw > 0;
        let b_has = b.wallet_balance_raw > 0 || b.supply_raw > 0
            || b.variable_debt_raw > 0 || b.stable_debt_raw > 0;
        b_has.cmp(&a_has)
            .then(b.supply_apr.partial_cmp(&a.supply_apr).unwrap_or(std::cmp::Ordering::Equal))
    });
    let display_scans = &display_scans[..display_scans.len().min(8)];

    let summaries: Vec<Value> = display_scans.iter().map(|s| {
        json!({
            "asset": s.asset,
            "symbol": s.symbol,
            "decimals": s.decimals,
            "a_token": s.a_token,
            "wallet_balance":     fmt_token_amount(s.wallet_balance_raw, s.decimals),
            "wallet_balance_raw": s.wallet_balance_raw.to_string(),
            "supply":             fmt_token_amount(s.supply_raw, s.decimals),
            "supply_raw":         s.supply_raw.to_string(),
            "variable_debt":      fmt_token_amount(s.variable_debt_raw, s.decimals),
            "stable_debt":        fmt_token_amount(s.stable_debt_raw, s.decimals),
            "used_as_collateral": s.used_as_collateral,
            "supply_apr_pct":     format!("{:.4}", s.supply_apr),
            "variable_borrow_apr_pct": format!("{:.4}", s.variable_borrow_apr),
            "stable_borrow_apr_pct":   format!("{:.4}", s.stable_borrow_apr),
        })
    }).collect();

    println!("{}", serde_json::to_string_pretty(&json!({
        "ok": true,
        "about": ABOUT,
        "chain": chain.key,
        "chain_id": chain.id,
        "chain_name": chain.name,
        "wallet": wallet,
        "rpc_failures": rpc_failures,
        "reserves_total": total_reserves,
        "native_balance":     fmt_token_amount(native_bal, 18),
        "native_balance_raw": native_bal.to_string(),
        "native_symbol": chain.native_symbol,
        "account": {
            "total_collateral_eth_1e18":  fmt_1e18(total_collateral_eth),
            "total_debt_eth_1e18":        fmt_1e18(total_debt_eth),
            "available_borrows_eth_1e18": fmt_1e18(available_borrows_eth),
            "health_factor_1e18":         if hf == u128::MAX { "infinite (no debt)".to_string() } else { fmt_1e18(hf) },
            "health_factor_raw": hf.to_string(),
            "note": "All ETH-equivalent values 1e18-scaled and oracle-priced (Aave V2 base unit is ETH on mainnet, USD on others). HF >= 1.0 healthy; < 1.0 liquidatable.",
        },
        "rewards_accrued":     fmt_token_amount(rewards_accrued, 18),
        "rewards_accrued_raw": rewards_accrued.to_string(),
        "rewards_token_note": "stkAAVE on Ethereum / WMATIC on Polygon / WAVAX on Avalanche. Run claim-rewards to harvest.",
        "status": status,
        "next_command": next_command,
        "tip": tip,
        "reserves_displayed": summaries.len(),
        "reserves": summaries,
        "note": format!("Aave V2 LendingPool on {} at {}. v0.1.0 enumerates ALL listed reserves at runtime via getReservesList(); displaying top {} by user activity / supply APR.", chain.name, chain.lending_pool, summaries.len()),
    }))?);
    Ok(())
}

async fn scan_reserve(asset: &str, chain: &ChainInfo, wallet: &str) -> anyhow::Result<ReserveScan> {
    // Step 1: get reserve metadata (rates + token addresses) from LendingPool
    let rd = lp_get_reserve_data(chain.lending_pool, asset, chain.rpc).await?;

    // Step 2: parallel fetch underlying symbol/decimals + user balances on a/s/v tokens
    let symbol_fut = erc20_symbol(asset, chain.rpc);
    let decimals_fut = erc20_decimals(asset, chain.rpc);
    let wallet_bal_fut = erc20_balance(asset, wallet, chain.rpc);
    let supply_fut = erc20_balance(&rd.a_token, wallet, chain.rpc);
    let v_debt_fut = erc20_balance(&rd.variable_debt_token, wallet, chain.rpc);
    let s_debt_fut = erc20_balance(&rd.stable_debt_token, wallet, chain.rpc);

    let (sym, dec, bal, supply_bal, v_debt_bal, s_debt_bal) = tokio::join!(
        symbol_fut, decimals_fut, wallet_bal_fut, supply_fut, v_debt_fut, s_debt_fut
    );

    Ok(ReserveScan {
        asset: asset.to_string(),
        symbol: sym,
        decimals: dec.unwrap_or(18),
        a_token: rd.a_token,
        s_debt_token: rd.stable_debt_token,
        v_debt_token: rd.variable_debt_token,
        wallet_balance_raw: bal.unwrap_or(0),
        supply_raw: supply_bal.unwrap_or(0),
        variable_debt_raw: v_debt_bal.unwrap_or(0),
        stable_debt_raw: s_debt_bal.unwrap_or(0),
        // V2 doesn't expose per-asset usageAsCollateralEnabled cheaply without PDP -
        // Aave's UI derives it from getUserConfiguration bitmap. v0.1.0 reports None;
        // borrow command uses overall account liquidity / HF for safety gating.
        used_as_collateral: false,
        supply_apr: ray_to_apr_pct(rd.current_liquidity_rate_ray),
        variable_borrow_apr: ray_to_apr_pct(rd.current_variable_borrow_rate_ray),
        stable_borrow_apr: ray_to_apr_pct(rd.current_stable_borrow_rate_ray),
    })
}

fn sensible_supply_amount(raw: u128, decimals: u32) -> String {
    let factor = 10u128.pow(decimals);
    let whole = raw / factor;
    let pick = whole.min(50).max(1);
    pick.to_string()
}
