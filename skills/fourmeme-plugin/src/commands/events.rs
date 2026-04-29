/// `fourmeme-plugin events --from-block X [--to-block Y] [--event TokenCreate|...]`
/// — fetch TokenManager V2 events via `eth_getLogs`.
///
/// Without `--event`, returns all 4 (TokenCreate / TokenPurchase / TokenSale /
/// LiquidityAdded). Block range is required because BSC `eth_getLogs` typically
/// caps at ~5000 blocks.

use anyhow::{Context, Result};
use clap::Args;
use serde_json::{json, Value};

use crate::config::{addresses, is_supported_chain, Urls};
use crate::rpc::parse_uint256_to_u128;

const TOPIC_TOKEN_CREATE:   &str = "0x396d5e902b675b032348d3d2e9517ee8f0c4a926603fbc075d3d282ff00cad20";
const TOPIC_TOKEN_PURCHASE: &str = "0x7db52723a3b2cdd6164364b3b766e65e540d7be48ffa89582956d8eaebe62942";
const TOPIC_TOKEN_SALE:     &str = "0x0a5575b3648bae2210cee56bf33254cc1ddfbc7bf637c0af2ac18b14fb1bae19";
const TOPIC_LIQUIDITY_ADDED:&str = "0xc18aa71171b358b706fe3dd345299685ba21a5316c66ffa9e319268b033c44b0";

#[derive(Args)]
pub struct EventsArgs {
    /// Block to start from (decimal or 0x-hex; "latest" rejected — pick a number)
    #[arg(long)]
    pub from_block: String,

    /// Optional end block (default: "latest")
    #[arg(long)]
    pub to_block: Option<String>,

    /// Filter to a single event: TokenCreate | TokenPurchase | TokenSale | LiquidityAdded
    #[arg(long)]
    pub event: Option<String>,

    #[arg(long, default_value_t = 56)]
    pub chain: u64,
}

pub async fn run(args: EventsArgs) -> Result<()> {
    match run_inner(args).await {
        Ok(()) => Ok(()),
        Err(e) => {
            println!("{}", super::error_response(&e, Some("events"), None));
            Ok(())
        }
    }
}

async fn run_inner(args: EventsArgs) -> Result<()> {
    if !is_supported_chain(args.chain) {
        anyhow::bail!("Chain {} not supported in v0.1.", args.chain);
    }

    fn block_to_hex(s: &str) -> Result<String> {
        let s = s.trim();
        if s == "latest" || s == "earliest" || s == "pending" {
            return Ok(s.to_string());
        }
        if let Some(stripped) = s.strip_prefix("0x") {
            u64::from_str_radix(stripped, 16).context("invalid hex block")?;
            return Ok(s.to_string());
        }
        let n: u64 = s.parse().with_context(|| format!("invalid block '{}'", s))?;
        Ok(format!("0x{:x}", n))
    }

    let from_block = block_to_hex(&args.from_block)?;
    let to_block = match args.to_block.as_deref() {
        None => "latest".to_string(),
        Some(s) => block_to_hex(s)?,
    };

    // Pick topic filter
    let topics_filter: Vec<&str> = match args.event.as_deref() {
        None => vec![TOPIC_TOKEN_CREATE, TOPIC_TOKEN_PURCHASE, TOPIC_TOKEN_SALE, TOPIC_LIQUIDITY_ADDED],
        Some("TokenCreate")   => vec![TOPIC_TOKEN_CREATE],
        Some("TokenPurchase") => vec![TOPIC_TOKEN_PURCHASE],
        Some("TokenSale")     => vec![TOPIC_TOKEN_SALE],
        Some("LiquidityAdded")=> vec![TOPIC_LIQUIDITY_ADDED],
        Some(other) => anyhow::bail!(
            "Unknown event '{}'. One of TokenCreate, TokenPurchase, TokenSale, LiquidityAdded.",
            other
        ),
    };

    let rpc = Urls::rpc_for_chain(args.chain)
        .ok_or_else(|| anyhow::anyhow!("no RPC for chain {}", args.chain))?;
    let body = json!({
        "jsonrpc": "2.0",
        "method":  "eth_getLogs",
        "params": [{
            "address":   addresses::TOKEN_MANAGER_V2,
            "fromBlock": from_block,
            "toBlock":   to_block,
            "topics":    [topics_filter],   // single OR-filter on topic[0]
        }],
        "id": 1
    });
    let resp = reqwest::Client::new().post(&rpc)
        .json(&body).send().await
        .context("eth_getLogs failed")?;
    let v: Value = resp.json().await.context("parsing getLogs response")?;
    if let Some(err) = v.get("error") {
        anyhow::bail!("RPC error: {}", err);
    }
    let logs = v["result"].as_array().cloned().unwrap_or_default();

    let decoded: Vec<Value> = logs.iter().map(|log| {
        let topics: Vec<&str> = log["topics"].as_array().map(|arr|
            arr.iter().filter_map(|v| v.as_str()).collect()
        ).unwrap_or_default();
        let topic0 = topics.first().copied().unwrap_or("");
        let event_name = match topic0 {
            t if t.eq_ignore_ascii_case(TOPIC_TOKEN_CREATE)    => "TokenCreate",
            t if t.eq_ignore_ascii_case(TOPIC_TOKEN_PURCHASE)  => "TokenPurchase",
            t if t.eq_ignore_ascii_case(TOPIC_TOKEN_SALE)      => "TokenSale",
            t if t.eq_ignore_ascii_case(TOPIC_LIQUIDITY_ADDED) => "LiquidityAdded",
            _ => "Unknown",
        };
        let block = log["blockNumber"].as_str()
            .map(|s| parse_uint256_to_u128(s).to_string())
            .unwrap_or_default();
        json!({
            "event":            event_name,
            "block_number":     block,
            "transaction_hash": log["transactionHash"],
            "log_index":        log["logIndex"],
            "topics":           log["topics"],
            "data":             log["data"],
        })
    }).collect();

    println!("{}", serde_json::to_string_pretty(&json!({
        "ok": true,
        "data": {
            "chain": "bsc",
            "chain_id": args.chain,
            "address": addresses::TOKEN_MANAGER_V2,
            "from_block": from_block,
            "to_block":   to_block,
            "event_filter": args.event.unwrap_or_else(|| "ALL".into()),
            "count": decoded.len(),
            "events": decoded,
            "tip": "Logs are returned with raw indexed topics + non-indexed data. \
                   Decode via the canonical event signatures listed in code (events.rs).",
        }
    }))?);
    Ok(())
}
