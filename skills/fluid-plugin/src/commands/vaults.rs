use clap::Args;
use crate::chain::{CHAIN_ETH, chain_name};
use crate::vault::{all_vault_addresses, vault_infos_batch, vault_type};
use crate::token::token_infos;

#[derive(Args)]
pub struct VaultsArgs {
    /// Chain ID (1 = Ethereum, 42161 = Arbitrum)
    #[arg(long, default_value_t = CHAIN_ETH)]
    pub chain: u64,
    /// Show all vault types (default: T1 only)
    #[arg(long)]
    pub all: bool,
    /// Maximum vaults to display
    #[arg(long, default_value_t = 30)]
    pub limit: usize,
}

pub async fn run(args: VaultsArgs) -> anyhow::Result<()> {
    eprintln!("[fluid] Fetching vault list from {} ({})...", chain_name(args.chain), args.chain);

    let all_addrs = all_vault_addresses(args.chain).await?;
    let total = all_addrs.len();

    // Fetch all vault data in a single batch call
    let all_infos = vault_infos_batch(args.chain, &all_addrs).await?;

    // Filter
    let infos: Vec<_> = all_infos.iter().filter(|v| {
        if args.all { true } else { !v.is_smart_col && !v.is_smart_debt }
    }).take(args.limit).collect();

    if infos.is_empty() {
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({
            "chain": args.chain,
            "vaults": [],
            "note": "No vaults found. Try --all to include smart vaults."
        }))?);
        return Ok(());
    }

    // Collect unique token addresses
    let token_addrs: Vec<String> = {
        let mut seen = std::collections::HashSet::new();
        let mut v = Vec::new();
        for info in &infos {
            for addr in [&info.col_token, &info.debt_token] {
                if seen.insert(addr.clone()) { v.push(addr.clone()); }
            }
        }
        v
    };
    let tokens = token_infos(args.chain, &token_addrs).await;

    let vault_list: Vec<serde_json::Value> = infos.iter().map(|v| {
        let col_sym = tokens.get(&v.col_token).map(|t| t.symbol.as_str()).unwrap_or("?");
        let debt_sym = tokens.get(&v.debt_token).map(|t| t.symbol.as_str()).unwrap_or("?");
        serde_json::json!({
            "vault":     v.address,
            "pair":      format!("{}/{}", col_sym, debt_sym),
            "col_token": v.col_token,
            "debt_token": v.debt_token,
            "type":      vault_type(v),
        })
    }).collect();

    let out = serde_json::json!({
        "chain":        args.chain,
        "chain_name":   chain_name(args.chain),
        "showing":      vault_list.len(),
        "total_vaults": total,
        "filter":       if args.all { "all" } else { "T1 only" },
        "vaults":       vault_list,
    });
    println!("{}", serde_json::to_string_pretty(&out)?);
    Ok(())
}
