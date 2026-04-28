use clap::Args;
use serde_json::{json, Value};

use crate::config::{ETH_KNOWN_MARKETS, SUPPORTED_CHAINS};
use crate::onchainos::resolve_wallet;
use crate::rpc::{
    balance_of_underlying, borrow_balance_current, borrow_rate_per_block, fmt_token_amount,
    get_account_liquidity, get_assets_in, get_comp_accrued, rate_per_block_to_apr,
    supply_rate_per_block,
};

#[derive(Args)]
pub struct PositionsArgs {
    /// Wallet address (default: onchainos wallet)
    #[arg(long)]
    pub address: Option<String>,
}

pub async fn run(args: PositionsArgs) -> anyhow::Result<()> {
    let chain = &SUPPORTED_CHAINS[0];

    let wallet = match args.address {
        Some(a) => a,
        None => match resolve_wallet(chain.id) {
            Ok(a) => a,
            Err(e) => {
                println!("{}", super::error_response(
                    &format!("{:#}", e), "WALLET_NOT_FOUND",
                    "Run `onchainos wallet addresses` to verify login or pass --address.",
                ));
                return Ok(());
            }
        },
    };

    let liquidity_fut = get_account_liquidity(chain.comptroller, &wallet, chain.rpc);
    let assets_fut = get_assets_in(chain.comptroller, &wallet, chain.rpc);
    let comp_fut = get_comp_accrued(chain.comptroller, &wallet, chain.rpc);
    let market_futs: Vec<_> = ETH_KNOWN_MARKETS.iter().map(|info| {
        let chain = chain.clone();
        let wallet = wallet.clone();
        async move {
            let supply_fut = balance_of_underlying(info.ctoken, &wallet, chain.rpc);
            let borrow_fut = borrow_balance_current(info.ctoken, &wallet, chain.rpc);
            let supply_rate = supply_rate_per_block(info.ctoken, chain.rpc);
            let borrow_rate = borrow_rate_per_block(info.ctoken, chain.rpc);
            let (s, b, sr, br) = tokio::join!(supply_fut, borrow_fut, supply_rate, borrow_rate);
            (info, s.ok(), b.ok(), sr.ok(), br.ok())
        }
    }).collect();

    let (liquidity_res, assets_res, comp_res, market_results) = tokio::join!(
        liquidity_fut, assets_fut, comp_fut, futures::future::join_all(market_futs)
    );

    let (err, liq, shortfall) = liquidity_res.unwrap_or((0, 0, 0));
    let assets_in: Vec<String> = assets_res.unwrap_or_default();
    let comp_accrued = comp_res.unwrap_or(0);

    let mut entries: Vec<Value> = Vec::new();
    for (info, supply, borrow, sr, br) in market_results {
        let supply_v = supply.unwrap_or(0);
        let borrow_v = borrow.unwrap_or(0);
        if supply_v == 0 && borrow_v == 0 { continue; }
        let supply_apr = sr.map(|r| rate_per_block_to_apr(r, chain.blocks_per_year));
        let borrow_apr = br.map(|r| rate_per_block_to_apr(r, chain.blocks_per_year));
        let entered_as_collateral = assets_in.iter().any(|a| a.eq_ignore_ascii_case(info.ctoken));
        entries.push(json!({
            "ctoken": info.ctoken,
            "ctoken_symbol": info.symbol,
            "underlying": info.underlying_symbol,
            "supply_underlying":      fmt_token_amount(supply_v, info.underlying_decimals),
            "supply_underlying_raw":  supply_v.to_string(),
            "borrow_underlying":      fmt_token_amount(borrow_v, info.underlying_decimals),
            "borrow_underlying_raw":  borrow_v.to_string(),
            "supply_apr_pct": supply_apr.map(|a| format!("{:.4}", a * 100.0)),
            "borrow_apr_pct": borrow_apr.map(|a| format!("{:.4}", a * 100.0)),
            "entered_as_collateral": entered_as_collateral,
        }));
    }

    println!("{}", serde_json::to_string_pretty(&json!({
        "ok": true,
        "chain": chain.key,
        "chain_id": chain.id,
        "wallet": wallet,
        "account_liquidity": {
            "error_code": err,
            "liquidity_usd_1e18":      fmt_token_amount(liq, 18),
            "liquidity_usd_raw":       liq.to_string(),
            "shortfall_usd_1e18":      fmt_token_amount(shortfall, 18),
            "shortfall_usd_raw":       shortfall.to_string(),
            "note": "1e18-scaled USD; non-zero shortfall = under-collateralized (liquidatable). 0 means safe."
        },
        "assets_in_count": assets_in.len(),
        "assets_in": assets_in,
        "comp_accrued":     fmt_token_amount(comp_accrued, 18),
        "comp_accrued_raw": comp_accrued.to_string(),
        "comp_accrued_note": "Stored value (not auto-accrued — actual claimable may be slightly higher after Comptroller.distributeSupplierComp triggered by claimComp).",
        "position_count": entries.len(),
        "positions": entries,
    }))?);
    Ok(())
}
