use clap::Args;
use crate::abi::{selector, calldata, encode_uint256, encode_int256, encode_address, parse_amount};
use crate::chain::{CHAIN_ETH, chain_name};
use crate::contracts::NATIVE_ETH;
use crate::onchainos::{resolve_wallet, wallet_contract_call, extract_tx_hash};
use crate::token::token_infos;
use crate::vault::vault_info_single;

#[derive(Args)]
pub struct SupplyArgs {
    /// Chain ID (1 = Ethereum, 42161 = Arbitrum)
    #[arg(long, default_value_t = CHAIN_ETH)]
    pub chain: u64,
    /// Vault address to supply into
    #[arg(long)]
    pub vault: String,
    /// Amount of collateral to supply (human-readable, e.g. 1.5 or 1000)
    #[arg(long)]
    pub amount: String,
    /// Existing NFT position ID to add collateral to (0 or omit to open a new position)
    #[arg(long, default_value_t = 0)]
    pub nft_id: u64,
    /// Wallet address (defaults to active onchainos wallet)
    #[arg(long)]
    pub wallet: Option<String>,
    /// Simulate without broadcasting (returns stub hashes)
    #[arg(long)]
    pub dry_run: bool,
    /// Broadcast the transaction (required to execute)
    #[arg(long)]
    pub confirm: bool,
}

pub async fn run(args: SupplyArgs) -> anyhow::Result<()> {
    let wallet = match &args.wallet {
        Some(w) => w.clone(),
        None => resolve_wallet(args.chain)?,
    };

    let vault_info = vault_info_single(args.chain, &args.vault).await?;
    let token_addrs = vec![vault_info.col_token.clone()];
    let tokens = token_infos(args.chain, &token_addrs).await;

    let col_tok = tokens.get(&vault_info.col_token);
    let col_dec = col_tok.map(|t| t.decimals).unwrap_or(18);
    let col_sym = col_tok.map(|t| t.symbol.as_str()).unwrap_or("?").to_string();
    let is_native = vault_info.col_token.to_lowercase() == NATIVE_ETH.to_lowercase();

    let col_raw = parse_amount(&args.amount, col_dec)?;
    if col_raw == 0 {
        anyhow::bail!("Supply amount must be greater than 0");
    }

    eprintln!("[fluid] Supply {} {} into vault {} on {}...",
        args.amount, col_sym, args.vault, chain_name(args.chain));

    // Build operate(nft_id, +newCol, 0, wallet) calldata
    let op_sel = selector("operate(uint256,int256,int256,address)");
    let op_data = calldata(op_sel, &[
        encode_uint256(args.nft_id as u128),
        encode_int256(col_raw as i128),
        encode_int256(0),
        encode_address(&wallet),
    ]);

    let approval_note = if is_native {
        "Native ETH — no approval needed"
    } else {
        "ERC-20 — approval tx will fire first"
    };

    let preview = serde_json::json!({
        "preview": true,
        "action": "supply",
        "vault": args.vault,
        "nft_id": args.nft_id,
        "col_token": vault_info.col_token,
        "col_symbol": col_sym,
        "amount": args.amount,
        "amount_raw": col_raw.to_string(),
        "wallet": wallet,
        "chain": args.chain,
        "note": approval_note,
        "confirm_hint": "Add --confirm to broadcast"
    });

    if !args.confirm && !args.dry_run {
        println!("{}", serde_json::to_string_pretty(&preview)?);
        return Ok(());
    }

    let mut approve_hash: Option<String> = None;

    // Approve ERC-20 collateral token for the vault
    if !is_native {
        let approve_sel = selector("approve(address,uint256)");
        let approve_data = calldata(approve_sel, &[
            encode_address(&args.vault),
            encode_uint256(col_raw),
        ]);
        eprintln!("[fluid] Approving {} {} for vault {}...", args.amount, col_sym, args.vault);
        let approve_resp = wallet_contract_call(
            args.chain, &vault_info.col_token, &approve_data, "0", args.dry_run, Some(&wallet),
        )?;
        approve_hash = Some(extract_tx_hash(&approve_resp));
        eprintln!("[fluid] Approval tx: {}", approve_hash.as_deref().unwrap_or("?"));
    }

    // Call operate — for native ETH, pass col_raw as --amt
    let amt_eth = if is_native { col_raw.to_string() } else { "0".to_string() };
    let resp = wallet_contract_call(
        args.chain, &args.vault, &op_data, &amt_eth, args.dry_run, Some(&wallet),
    )?;
    let tx_hash = extract_tx_hash(&resp);

    let out = serde_json::json!({
        "ok": true,
        "action": "supply",
        "vault": args.vault,
        "nft_id": args.nft_id,
        "col_symbol": col_sym,
        "amount": args.amount,
        "approve_tx_hash": approve_hash,
        "tx_hash": tx_hash,
        "wallet": wallet,
        "chain": args.chain,
    });
    println!("{}", serde_json::to_string_pretty(&out)?);
    Ok(())
}
