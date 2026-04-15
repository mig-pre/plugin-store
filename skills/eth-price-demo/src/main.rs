use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Parser)]
#[command(
    name = "eth-price-demo",
    version,
    about = "Query ETH price via OnchainOS"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Get current ETH price in USD
    GetPrice {
        /// Chain ID (default: 1 for Ethereum mainnet)
        #[arg(long, default_value = "1")]
        chain: String,
    },
}

#[derive(Debug, Deserialize)]
struct OkxTickerResponse {
    code: String,
    data: Vec<OkxTicker>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OkxTicker {
    last: String,
    #[serde(rename = "open24h")]
    open_24h: String,
    #[serde(rename = "high24h")]
    high_24h: String,
    #[serde(rename = "low24h")]
    low_24h: String,
    #[serde(rename = "volCcy24h")]
    vol_ccy_24h: String,
}

#[derive(Serialize)]
struct PriceOutput {
    ok: bool,
    token: String,
    chain_id: String,
    price_usd: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    open_24h: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    high_24h: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    low_24h: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    volume_24h: Option<String>,
    source: String,
}

/// Try to get ETH price via onchainos CLI first, fall back to OKX public API.
async fn get_eth_price(chain: &str) -> Result<()> {
    // Attempt 1: onchainos CLI
    if let Ok(output) = try_onchainos(chain) {
        println!("{}", output);
        return Ok(());
    }

    // Attempt 2: OKX public REST API
    let result = try_okx_api(chain).await?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

fn try_onchainos(chain: &str) -> Result<String> {
    let output = Command::new("onchainos")
        .args([
            "dex",
            "token",
            "price-info",
            "--chain",
            chain,
            "--token",
            "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE",
        ])
        .output()
        .context("onchainos not found")?;

    if !output.status.success() {
        anyhow::bail!("onchainos returned non-zero exit code");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.trim().is_empty() {
        anyhow::bail!("onchainos returned empty output");
    }

    // Parse and re-format for consistent output
    let json: serde_json::Value =
        serde_json::from_str(&stdout).context("failed to parse onchainos output")?;
    Ok(serde_json::to_string_pretty(&json)?)
}

async fn try_okx_api(chain: &str) -> Result<PriceOutput> {
    let client = reqwest::Client::new();
    let resp: OkxTickerResponse = client
        .get("https://www.okx.com/api/v5/market/ticker?instId=ETH-USDT")
        .send()
        .await
        .context("failed to call OKX API")?
        .json()
        .await
        .context("failed to parse OKX response")?;

    if resp.code != "0" || resp.data.is_empty() {
        anyhow::bail!("OKX API returned error code: {}", resp.code);
    }

    let ticker = &resp.data[0];
    Ok(PriceOutput {
        ok: true,
        token: "ETH".to_string(),
        chain_id: chain.to_string(),
        price_usd: ticker.last.clone(),
        open_24h: Some(ticker.open_24h.clone()),
        high_24h: Some(ticker.high_24h.clone()),
        low_24h: Some(ticker.low_24h.clone()),
        volume_24h: Some(ticker.vol_ccy_24h.clone()),
        source: "okx-public-api".to_string(),
    })
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::GetPrice { chain } => get_eth_price(&chain).await?,
    }
    Ok(())
}
