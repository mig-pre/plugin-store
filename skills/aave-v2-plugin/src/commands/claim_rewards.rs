use clap::Args;
use serde_json::json;

use crate::config::{parse_chain, supported_chains_help, ChainInfo, SUPPORTED_CHAINS};
use crate::onchainos::{extract_tx_hash, resolve_wallet, wallet_contract_call};
use crate::rpc::{
    erc20_balance, fmt_token_amount, get_reserves_list, incentives_get_unclaimed_rewards,
    incentives_reward_token, lp_get_reserve_data, native_balance, pad_address, pad_u256,
    pad_u256_max, selectors, wait_for_tx,
};

/// Claim accrued rewards from Aave V2 IncentivesController.
///
/// Reward token varies by chain:
///   - Ethereum: stkAAVE (Aave staking token; can be unstaked to AAVE after cooldown)
///   - Polygon V2:   WMATIC
///   - Avalanche V2: WAVAX
///
/// Implementation: claimRewards(address[] assets, uint256 amount, address to). Pass
/// uint256.max for `amount` to claim everything; the controller caps internally.
/// `assets` parameter is the union of all aTokens + s/v debt tokens for reserves the
/// user has activity in (we just pass all aTokens + s/v debt for ALL reserves on the
/// chain - controller ignores zero-balance ones).
///
/// All operations require explicit `--confirm`.
#[derive(Args)]
pub struct ClaimRewardsArgs {
    /// Chain key or id (ETH / POLYGON / AVAX).
    #[arg(long, default_value = "ETH")]
    pub chain: String,
    #[arg(long)]
    pub dry_run: bool,
    #[arg(long)]
    pub confirm: bool,
    #[arg(long, default_value = "300")]
    pub timeout_secs: u64,
}

pub async fn run(args: ClaimRewardsArgs) -> anyhow::Result<()> {
    let chain: &ChainInfo = match parse_chain(&args.chain) {
        Some(c) => c,
        None => return print_err(
            &format!("Unknown --chain '{}'", args.chain),
            "INVALID_CHAIN",
            &format!("Supported: {}", supported_chains_help()),
        ),
    };

    if chain.incentives_controller.is_empty() {
        return print_err(
            &format!("No active rewards program for Aave V2 on {}", chain.key),
            "NO_REWARDS_CONTROLLER",
            "This chain has no IncentivesController in v0.1.0 config.",
        );
    }

    let from_addr = match resolve_wallet(chain.id) {
        Ok(a) => a,
        Err(e) => return print_err(&format!("{:#}", e), "WALLET_NOT_FOUND",
            "Run `onchainos wallet addresses`."),
    };

    // Pre-flight: gas + accrued rewards
    let native = native_balance(&from_addr, chain.rpc).await
        .map_err(|e| anyhow::anyhow!("RPC: {}", e))?;
    if native < chain.gas_floor_wei {
        return print_err(
            &format!("Native {} insufficient on {}", chain.native_symbol, chain.key),
            "INSUFFICIENT_GAS", "Top up native gas.",
        );
    }

    let unclaimed = incentives_get_unclaimed_rewards(chain.incentives_controller, &from_addr, chain.rpc)
        .await.unwrap_or(0);

    let reward_token = incentives_reward_token(chain.incentives_controller, chain.rpc).await
        .unwrap_or_else(|_| "0x0000000000000000000000000000000000000000".to_string());
    let reward_token_balance_before = if reward_token != "0x0000000000000000000000000000000000000000" {
        erc20_balance(&reward_token, &from_addr, chain.rpc).await.unwrap_or(0)
    } else { 0 };

    // Build assets[] = aToken + sDebt + vDebt for ALL reserves (controller ignores zero)
    eprintln!("[claim-rewards] Enumerating reserves to build assets[] for claimRewards...");
    let reserves = match get_reserves_list(chain.lending_pool, chain.rpc).await {
        Ok(r) => r,
        Err(e) => return print_err(
            &format!("getReservesList: {:#}", e), "RPC_ERROR",
            "Public RPC may be limited; retry shortly.",
        ),
    };

    let token_futs: Vec<_> = reserves.iter().map(|asset| {
        let chain = chain.clone();
        let asset = asset.clone();
        async move { lp_get_reserve_data(chain.lending_pool, &asset, chain.rpc).await }
    }).collect();
    let token_results = futures::future::join_all(token_futs).await;

    let mut assets: Vec<String> = Vec::new();
    let zero_addr = "0x0000000000000000000000000000000000000000";
    for r in token_results {
        if let Ok(rd) = r {
            if !rd.a_token.is_empty() && rd.a_token != zero_addr { assets.push(rd.a_token); }
            if !rd.stable_debt_token.is_empty() && rd.stable_debt_token != zero_addr { assets.push(rd.stable_debt_token); }
            if !rd.variable_debt_token.is_empty() && rd.variable_debt_token != zero_addr { assets.push(rd.variable_debt_token); }
        }
    }
    if assets.is_empty() {
        return print_err(
            "No claimable assets resolved.", "NO_ASSETS",
            "Reserves may have failed to enumerate. Retry.",
        );
    }

    // Build calldata: claimRewards(address[] assets, uint256 amount, address to)
    // ABI: selector + offset_to_array(0x60) + amount + to + array_length + addr × N
    let mut calldata = String::new();
    calldata.push_str(selectors::CLAIM_REWARDS);
    calldata.push_str(&pad_u256(0x60));                       // offset to assets array
    calldata.push_str(&pad_u256_max());                       // amount = uint256.max (claim all)
    calldata.push_str(&pad_address(&from_addr));              // to
    calldata.push_str(&pad_u256(assets.len() as u128));       // array length
    for a in &assets {
        calldata.push_str(&pad_address(a));
    }

    let stage = if args.dry_run { "dry_run" } else if args.confirm { "submit" } else { "preview" };
    println!("{}", serde_json::to_string_pretty(&json!({
        "ok": true,
        "stage": stage,
        "submitted": false,
        "preview": {
            "action": "claim_rewards",
            "chain": chain.key,
            "controller": chain.incentives_controller,
            "reward_token": reward_token,
            "user": from_addr,
            "assets_count": assets.len(),
            "unclaimed_pre_distribute":     fmt_token_amount(unclaimed, 18),
            "unclaimed_pre_distribute_raw": unclaimed.to_string(),
            "reward_token_balance_before":  fmt_token_amount(reward_token_balance_before, 18),
            "amount_to_claim": "uint256.max (claim all)",
            "note": "Stored compAccrued underestimates actual claimable; claimRewards triggers distribution settlement first, then transfers full accrued amount.",
        }
    }))?);

    if args.dry_run { eprintln!("[DRY RUN]"); return Ok(()); }
    if !args.confirm { eprintln!("[PREVIEW] Add --confirm to submit."); return Ok(()); }

    // Gas: per-asset distribution iteration; conservative 200k + 30k * N
    let gas_limit = 200_000u64 + (assets.len() as u64) * 30_000;
    eprintln!("[claim-rewards] Submitting claimRewards({} assets)...", assets.len());
    let result = match wallet_contract_call(chain.id, chain.incentives_controller, &calldata, None, Some(gas_limit), false) {
        Ok(r) => r,
        Err(e) => return print_err(&format!("claimRewards failed: {:#}", e),
            "CLAIM_FAILED", "Common: gas, RPC, controller paused."),
    };
    let tx_hash = extract_tx_hash(&result);

    match tx_hash.as_ref() {
        Some(h) => {
            eprintln!("[claim-rewards] Submit tx: {} - waiting...", h);
            if let Err(e) = wait_for_tx(h, chain.rpc, args.timeout_secs).await {
                return print_err(&format!("Tx {} reverted: {:#}", h, e),
                    "TX_REVERTED", "On-chain revert. Inspect on the block explorer.");
            }
            eprintln!("[claim-rewards] On-chain confirmed.");
        }
        None => return print_err("claimRewards broadcast but no tx hash",
            "TX_HASH_MISSING", "Check `onchainos wallet history`."),
    }

    let reward_token_balance_after = if reward_token != "0x0000000000000000000000000000000000000000" {
        erc20_balance(&reward_token, &from_addr, chain.rpc).await.unwrap_or(0)
    } else { 0 };
    let claimed = reward_token_balance_after.saturating_sub(reward_token_balance_before);

    println!("{}", serde_json::to_string_pretty(&json!({
        "ok": true,
        "action": "claim_rewards",
        "chain": chain.key,
        "user": from_addr,
        "reward_token": reward_token,
        "reward_token_balance_before": fmt_token_amount(reward_token_balance_before, 18),
        "reward_token_balance_after":  fmt_token_amount(reward_token_balance_after, 18),
        "claimed":     fmt_token_amount(claimed, 18),
        "claimed_raw": claimed.to_string(),
        "tx_hash": tx_hash,
        "on_chain_status": "0x1",
        "tip": "Reward token now in your wallet. On Ethereum: stkAAVE has a 10-day unstake cooldown to convert to AAVE. On Polygon/Avalanche: rewards are wrapped native (WMATIC/WAVAX) - unwrap to native externally.",
    }))?);
    Ok(())
}

fn print_err(msg: &str, code: &str, suggestion: &str) -> anyhow::Result<()> {
    println!("{}", super::error_response(msg, code, suggestion));
    Ok(())
}
