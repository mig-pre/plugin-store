use clap::Args;
use serde_json::{json, Value};

use crate::config::{ChainInfo, ConvertMechanism, SUPPORTED_CHAINS, STABLE_DECIMALS, parse_chain, supported_chains_help};
use crate::onchainos::resolve_wallet;
use crate::rpc::{erc20_balance, fmt_token_amount, native_balance, vault_convert_to_assets};

#[derive(Args)]
pub struct BalanceArgs {
    /// Wallet address to query. Defaults to onchainos wallet.
    #[arg(long)]
    pub address: Option<String>,
    /// Single chain (id or key). If omitted, queries all 3 supported chains.
    #[arg(long)]
    pub chain: Option<String>,
}

pub async fn run(args: BalanceArgs) -> anyhow::Result<()> {
    let chains: Vec<&'static ChainInfo> = if let Some(s) = &args.chain {
        match parse_chain(s) {
            Some(c) => vec![c],
            None => {
                println!("{}", super::error_response(
                    &format!("Unsupported chain '{}'", s),
                    "UNSUPPORTED_CHAIN",
                    &format!("Use one of: {}", supported_chains_help()),
                ));
                return Ok(());
            }
        }
    } else {
        SUPPORTED_CHAINS.iter().collect()
    };

    let mut entries: Vec<Value> = Vec::with_capacity(chains.len());
    let mut total_susds_raw: u128 = 0;
    let mut total_underlying_usds_raw: u128 = 0;

    for chain in chains {
        let address = match args.address.clone() {
            Some(a) => a,
            None => match resolve_wallet(chain.id) {
                Ok(a) => a,
                Err(e) => {
                    entries.push(json!({
                        "chain": chain.key,
                        "chain_id": chain.id,
                        "error": format!("wallet resolve failed: {:#}", e),
                        "error_code": "WALLET_NOT_FOUND",
                    }));
                    continue;
                }
            },
        };

        let native_fut = native_balance(&address, chain.rpc);
        let usds_fut = erc20_balance(chain.usds, &address, chain.rpc);
        let susds_fut = erc20_balance(chain.susds, &address, chain.rpc);

        let (n, u, s) = tokio::join!(native_fut, usds_fut, susds_fut);

        let native_raw = match n {
            Ok(v) => v,
            Err(e) => {
                entries.push(json!({
                    "chain": chain.key, "chain_id": chain.id, "address": address,
                    "error": format!("native: {}", e), "error_code": "RPC_ERROR",
                }));
                continue;
            }
        };
        let usds_raw = u.unwrap_or(0);
        let susds_raw = s.unwrap_or(0);

        // Convert sUSDS shares to underlying USDS value (only Ethereum has the
        // ERC-4626 convertToAssets; on L2 sUSDS, the rate-provider oracle is
        // not reliably callable, so we fall back to assuming 1:1 + a note).
        let (susds_in_usds, susds_value_method) = match chain.mechanism {
            ConvertMechanism::Erc4626Vault if susds_raw > 0 => {
                match vault_convert_to_assets(chain.susds, susds_raw, chain.rpc).await {
                    Ok(v) => (v, "ERC-4626 convertToAssets"),
                    Err(_) => (susds_raw, "fallback 1:1 (RPC failed)"),
                }
            }
            _ => (susds_raw, "approx 1:1 (cross-chain sUSDS, not a vault)"),
        };

        // Optional: DAI on Ethereum
        let mut entry = json!({
            "chain": chain.key,
            "chain_id": chain.id,
            "mechanism": match chain.mechanism {
                ConvertMechanism::Erc4626Vault => "erc4626_vault",
                ConvertMechanism::SparkPsm     => "spark_psm",
            },
            "address": address,
            "native": {
                "symbol": chain.native_symbol,
                "amount": fmt_token_amount(native_raw, 18),
                "amount_raw": native_raw.to_string(),
            },
            "usds": {
                "amount": fmt_token_amount(usds_raw, STABLE_DECIMALS),
                "amount_raw": usds_raw.to_string(),
            },
            "susds": {
                "amount": fmt_token_amount(susds_raw, STABLE_DECIMALS),
                "amount_raw": susds_raw.to_string(),
                "underlying_usds": fmt_token_amount(susds_in_usds, STABLE_DECIMALS),
                "underlying_usds_raw": susds_in_usds.to_string(),
                "valuation_method": susds_value_method,
            },
        });

        if let Some(dai_addr) = chain.dai {
            let dai_raw = erc20_balance(dai_addr, &address, chain.rpc).await.unwrap_or(0);
            entry["dai"] = json!({
                "amount": fmt_token_amount(dai_raw, STABLE_DECIMALS),
                "amount_raw": dai_raw.to_string(),
                "tip": if dai_raw > 0 {
                    "Use `spark-savings-plugin upgrade-dai --amount <X> --confirm` to convert legacy DAI 1:1 to USDS, then deposit."
                } else { "" },
            });
        }

        total_susds_raw = total_susds_raw.saturating_add(susds_raw);
        total_underlying_usds_raw = total_underlying_usds_raw.saturating_add(susds_in_usds);

        entries.push(entry);
    }

    println!("{}", serde_json::to_string_pretty(&json!({
        "ok": true,
        "count": entries.len(),
        "total_susds_across_chains": fmt_token_amount(total_susds_raw, STABLE_DECIMALS),
        "total_susds_across_chains_raw": total_susds_raw.to_string(),
        "total_underlying_usds_across_chains": fmt_token_amount(total_underlying_usds_raw, STABLE_DECIMALS),
        "total_underlying_usds_across_chains_raw": total_underlying_usds_raw.to_string(),
        "balances": entries,
    }))?);
    Ok(())
}
