/// `polymarket deposit` — fund the proxy wallet.
///
/// ## Default path (Polygon, direct)
/// Sends an ERC-20 USDC.e transfer from the onchainos EOA wallet directly to
/// the proxy wallet on Polygon (chain 137). No bridge involved.
///
/// ## Bridge path (other EVM chains)
/// For chains where onchainos can sign (ETH/ARB/BASE/OP/BNB/Monad), the command:
///   1. Gets a bridge deposit address (POST /deposit with proxy wallet)
///   2. Sends tokens from the EOA to the bridge deposit address via onchainos
///   3. Polls bridge status until COMPLETED
///
/// ## Manual path (chains onchainos cannot sign for)
/// For chains like BTC, Tron, Abstract, etc.:
///   1. Gets a bridge deposit address
///   2. Displays it for the user to send manually
///   3. Polls bridge status until COMPLETED
///
/// ## List mode
/// `--list` fetches GET /supported-assets and prints all chains + tokens.
///
/// Prerequisites: `polymarket setup-proxy` must have been run first.

use anyhow::{bail, Result};
use reqwest::Client;
use std::collections::HashMap;

/// EVM chain IDs that onchainos can send transactions on (besides Polygon=137).
/// Polygon is handled separately (direct transfer, no bridge).
const BRIDGE_ONCHAINOS_CHAIN_IDS: &[&str] = &[
    "1",     // Ethereum
    "42161", // Arbitrum
    "8453",  // Base
    "10",    // Optimism
    "56",    // BNB Chain
    "143",   // Monad
];

/// Map common user-input chain names to the chainId used by the bridge API.
fn resolve_chain_id(chain: &str) -> Option<&'static str> {
    match chain.to_lowercase().as_str() {
        "polygon" | "matic" | "137" => Some("137"),
        "ethereum" | "eth" | "1" => Some("1"),
        "arbitrum" | "arb" | "42161" => Some("42161"),
        "base" | "8453" => Some("8453"),
        "optimism" | "op" | "10" => Some("10"),
        "bnb" | "bsc" | "56" => Some("56"),
        "monad" | "143" => Some("143"),
        "bitcoin" | "btc" => Some("btc"),
        "tron" | "trx" => Some("tron"),
        "solana" | "sol" => Some("sol"),
        _ => None,
    }
}

/// Map bridge chainId → onchainos chain argument (name or numeric ID).
fn onchainos_chain_arg(chain_id: &str) -> &str {
    match chain_id {
        "1" => "ethereum",
        "42161" => "arbitrum",
        "8453" => "base",
        "10" => "optimism",
        "56" => "bnb",
        "143" => "monad",
        other => other,
    }
}

pub async fn run(
    amount: Option<&str>,
    chain: &str,
    token: &str,
    list: bool,
    dry_run: bool,
) -> Result<()> {
    let client = Client::new();

    // ── --list mode ─────────────────────────────────────────────────────────
    if list {
        let assets = crate::api::bridge_supported_assets(&client).await?;
        // Group by chain
        let mut by_chain: HashMap<String, Vec<&crate::api::BridgeAsset>> = HashMap::new();
        for a in &assets {
            by_chain
                .entry(format!("{} (chainId: {})", a.chain_name, a.chain_id))
                .or_default()
                .push(a);
        }
        let mut chains: Vec<_> = by_chain.keys().collect();
        chains.sort();
        let mut out = serde_json::json!({ "ok": true, "data": [] });
        let arr = out["data"].as_array_mut().unwrap();
        for chain_key in chains {
            let tokens = &by_chain[chain_key];
            let token_list: Vec<_> = tokens
                .iter()
                .map(|a| {
                    serde_json::json!({
                        "symbol": a.token.symbol,
                        "name": a.token.name,
                        "minUsd": a.min_checkout_usd,
                        "decimals": a.token.decimals,
                        "address": a.token.address,
                    })
                })
                .collect();
            arr.push(serde_json::json!({
                "chain": chain_key,
                "tokens": token_list,
            }));
        }
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }

    // ── Normal deposit — amount required ────────────────────────────────────
    // If amount is missing, return a structured response so the upstream Agent
    // knows exactly which parameters to ask the user for before retrying.
    let amount_str = match amount {
        Some(a) => a,
        None => {
            println!(
                "{}",
                serde_json::json!({
                    "ok": false,
                    "missing_params": ["amount"],
                    "error": "Missing required parameter: --amount (USD value to deposit, e.g. 50 = $50).",
                    "hint": "Please ask the user: how much USD do you want to deposit? \
                             Also confirm: which chain to deposit from (default: polygon) \
                             and which token to use (default: USDC)."
                })
            );
            return Ok(());
        }
    };

    let amount_f: f64 = amount_str
        .parse()
        .map_err(|_| anyhow::anyhow!("invalid amount: {}", amount_str))?;
    if amount_f <= 0.0 {
        bail!("amount must be positive");
    }

    let signer_addr = crate::onchainos::get_wallet_address().await?;
    let creds = crate::auth::ensure_credentials(&client, &signer_addr).await?;
    let proxy_wallet = creds.proxy_wallet.as_ref().ok_or_else(|| {
        anyhow::anyhow!("No proxy wallet configured. Run `polymarket setup-proxy` first.")
    })?;

    // ── Resolve chain ────────────────────────────────────────────────────────
    let chain_id = resolve_chain_id(chain).ok_or_else(|| {
        anyhow::anyhow!(
            "Unknown chain '{}'. Use --list to see supported chains, or try: polygon, ethereum, arbitrum, base, optimism, bnb, monad",
            chain
        )
    })?;

    // ── Polygon: direct ERC-20 transfer (no bridge) ─────────────────────────
    // --amount is USD. For USDC.e (6 decimals, 1:1 USD), amount_raw = amount_f × 1e6.
    if chain_id == "137" {
        // Only USDC.e is supported for direct deposit
        let token_upper = token.to_uppercase();
        if !matches!(token_upper.as_str(), "USDC.E" | "USDC" | "USDCE") {
            bail!(
                "Direct Polygon deposit only supports USDC.e. \
                 Use --chain ethereum (or arbitrum/base/op/bnb) to deposit other tokens via bridge."
            );
        }
        let amount_raw = (amount_f * 1_000_000.0).round() as u128;

        if dry_run {
            println!(
                "{}",
                serde_json::json!({
                    "ok": true,
                    "dry_run": true,
                    "data": {
                        "chain": "polygon",
                        "from": signer_addr,
                        "to": proxy_wallet,
                        "token": "USDC.e",
                        "amount": amount_f,
                        "amount_raw": amount_raw,
                        "note": "dry-run: no transaction submitted"
                    }
                })
            );
            return Ok(());
        }

        eprintln!(
            "[polymarket] Transferring {} USDC.e to proxy wallet {} on Polygon...",
            amount_f, proxy_wallet
        );
        let tx_hash = crate::onchainos::transfer_usdc_to_proxy(proxy_wallet, amount_raw).await?;

        println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "data": {
                    "chain": "polygon",
                    "tx_hash": tx_hash,
                    "from": signer_addr,
                    "to": proxy_wallet,
                    "token": "USDC.e",
                    "amount": amount_f,
                    "note": "USDC.e deposited to proxy wallet."
                }
            })
        );
        return Ok(());
    }

    // ── Non-Polygon: bridge path ─────────────────────────────────────────────
    // Fetch supported assets to find token contract + validate chain/token combo
    let assets = crate::api::bridge_supported_assets(&client).await?;
    let token_upper = token.to_uppercase();

    // Find matching asset for this chain + token
    let asset = assets.iter().find(|a| {
        a.chain_id == chain_id
            && (a.token.symbol.to_uppercase() == token_upper
                || a.token.name.to_uppercase() == token_upper)
    });

    let asset = match asset {
        Some(a) => a,
        None => {
            // Show what IS available on this chain
            let available: Vec<_> = assets
                .iter()
                .filter(|a| a.chain_id == chain_id)
                .map(|a| a.token.symbol.as_str())
                .collect();
            if available.is_empty() {
                bail!(
                    "Chain '{}' (id: {}) is not supported by the bridge. \
                     Use `polymarket deposit --list` to see all supported chains.",
                    chain, chain_id
                );
            } else {
                bail!(
                    "Token '{}' not found on chain '{}'. Available tokens: {}",
                    token,
                    chain,
                    available.join(", ")
                );
            }
        }
    };

    // ── USD minimum check (BEFORE any on-chain action) ──────────────────────
    // --amount is always in USD. Convert to token quantity using live price for
    // non-stablecoins. Hard-fail here so the user never loses funds to a
    // below-minimum deposit that the bridge silently ignores.
    let min_usd = asset.min_checkout_usd;

    // Stablecoins: 1 token ≈ $1, no price fetch needed.
    let is_stablecoin = asset.token.decimals <= 6
        && matches!(
            asset.token.symbol.to_uppercase().as_str(),
            "USDC" | "USDC.E" | "USDCE" | "USDT" | "USDT0"
                | "USD\u{20AE}0" | "DAI" | "BUSD" | "USDP" | "PYUSD"
                | "USDS" | "USDE" | "USDG" | "MUSD" | "USDBC"
                | "EURC" | "EUROC" | "EUR24"
        );

    let token_price_usd: f64 = if is_stablecoin {
        1.0
    } else {
        // Fetch live price — must succeed before we touch the chain.
        eprintln!("[polymarket] Fetching {} price...", asset.token.symbol);
        match crate::api::get_token_price_usd(&client, chain_id, &asset.token.address).await {
            Some(p) => p,
            None => bail!(
                "Could not fetch USD price for {} on {}. \
                 Try depositing a stablecoin (USDC, USDT) instead, or retry.",
                asset.token.symbol, asset.chain_name
            ),
        }
    };

    // amount_f is USD → enforce minimum before going any further.
    if amount_f < min_usd {
        bail!(
            "Amount ${:.2} is below the bridge minimum ${:.2} for {} on {}. \
             Please deposit at least ${:.0}.",
            amount_f, min_usd, asset.token.symbol, asset.chain_name, min_usd
        );
    }

    // Convert USD amount → token raw units.
    let token_qty = amount_f / token_price_usd;
    let amount_raw = (token_qty * 10f64.powi(asset.token.decimals as i32)).round() as u128;
    if amount_raw == 0 {
        bail!("Computed token quantity is 0. Check the amount and token.");
    }
    let can_auto_send = BRIDGE_ONCHAINOS_CHAIN_IDS.contains(&chain_id);

    // Get bridge deposit address
    eprintln!("[polymarket] Getting bridge deposit address for proxy wallet {}...", proxy_wallet);
    let bridge_deposit_addr = crate::api::bridge_get_deposit_address(&client, proxy_wallet).await?;

    if dry_run {
        println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "dry_run": true,
                "data": {
                    "chain": asset.chain_name,
                    "chain_id": chain_id,
                    "token": asset.token.symbol,
                    "amount_usd": amount_f,
                    "token_qty": token_qty,
                    "token_price_usd": if is_stablecoin { serde_json::Value::Null } else { serde_json::json!(token_price_usd) },
                    "amount_raw": amount_raw,
                    "bridge_deposit_address": bridge_deposit_addr,
                    "from": signer_addr,
                    "auto_send": can_auto_send,
                    "note": "dry-run: no transaction submitted"
                }
            })
        );
        return Ok(());
    }

    let tx_hash: Option<String> = if can_auto_send {
        // onchainos can sign on this chain — send automatically
        let oc_chain = onchainos_chain_arg(chain_id);
        eprintln!(
            "[polymarket] Sending {} {} on {} → bridge deposit address {}...",
            amount_f, asset.token.symbol, asset.chain_name, bridge_deposit_addr
        );
        // ETH sentinel address means native coin — use native transfer instead of ERC-20
        let is_native = asset.token.address.to_lowercase()
            == "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
        let hash = if is_native {
            crate::onchainos::transfer_native_on_chain(
                oc_chain,
                &bridge_deposit_addr,
                amount_raw,
            )
            .await?
        } else {
            crate::onchainos::transfer_erc20_on_chain(
                oc_chain,
                &asset.token.address,
                &bridge_deposit_addr,
                amount_raw,
            )
            .await?
        };
        eprintln!("[polymarket] Sent. tx_hash: {}", hash);
        Some(hash)
    } else {
        // Manual chain (BTC, Tron, Solana, etc.) — show address, user sends manually
        println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "data": {
                    "chain": asset.chain_name,
                    "token": asset.token.symbol,
                    "amount": amount_f,
                    "bridge_deposit_address": bridge_deposit_addr,
                    "note": format!(
                        "Send exactly {} {} to the bridge deposit address above, then wait for confirmation.",
                        amount_f, asset.token.symbol
                    )
                }
            })
        );
        None
    };

    // ── Poll bridge status ───────────────────────────────────────────────────
    eprintln!("[polymarket] Waiting for bridge to process deposit...");
    let mut attempts = 0u32;
    let max_attempts = 60; // 5 minutes at 5s interval
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        attempts += 1;

        match crate::api::bridge_poll_status(&client, &bridge_deposit_addr).await {
            Ok(crate::api::BridgeStatus::Completed) => {
                println!(
                    "{}",
                    serde_json::json!({
                        "ok": true,
                        "data": {
                            "status": "COMPLETED",
                            "chain": asset.chain_name,
                            "token": asset.token.symbol,
                            "amount": amount_f,
                            "bridge_deposit_address": bridge_deposit_addr,
                            "tx_hash": tx_hash,
                            "proxy_wallet": proxy_wallet,
                            "note": "Deposit completed. Funds are now in your proxy wallet as USDC."
                        }
                    })
                );
                return Ok(());
            }
            Ok(crate::api::BridgeStatus::Failed) => {
                bail!(
                    "Bridge deposit FAILED. bridge_deposit_address: {}. \
                     Check the bridge status manually.",
                    bridge_deposit_addr
                );
            }
            Ok(crate::api::BridgeStatus::Pending(state)) => {
                eprintln!(
                    "[polymarket] Bridge status: {} (attempt {}/{})",
                    state, attempts, max_attempts
                );
                if attempts >= max_attempts {
                    bail!(
                        "Bridge deposit timed out after {} attempts. Last status: {}. \
                         bridge_deposit_address: {}",
                        max_attempts, state, bridge_deposit_addr
                    );
                }
            }
            Err(e) => {
                eprintln!("[polymarket] Bridge poll error (attempt {}): {}", attempts, e);
                if attempts >= max_attempts {
                    bail!("Bridge poll failed after {} attempts: {}", max_attempts, e);
                }
            }
        }
    }
}
