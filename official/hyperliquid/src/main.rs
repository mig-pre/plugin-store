mod auth;
mod client;

use anyhow::{bail, Context as _, Result};
use clap::{Parser, Subcommand};
use client::HyperliquidClient;
use serde_json::{json, Value};

// Hyperliquid Bridge2 on Arbitrum One
const HL_BRIDGE: &str = "0x2df1c51e09aecf9cacb7bc98cb1742757f163df7";
// Native USDC on Arbitrum One
const USDC_ARBITRUM: &str = "0xaf88d065e77c8cC2239327C5EDb3A432268e5831";
// Arbitrum One chain ID
const ARBITRUM_CHAIN_ID: &str = "42161";
// Public Arbitrum One JSON-RPC
const ARBITRUM_RPC: &str = "https://arb1.arbitrum.io/rpc";

fn output(v: Value) {
    println!("{}", serde_json::to_string_pretty(&v).unwrap_or_default());
}

#[derive(Parser)]
#[command(name = "dapp-hyperliquid", about = "Hyperliquid perpetual & spot trading CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// List perpetual futures markets
    Markets,
    /// List spot markets
    SpotMarkets,
    /// Get real-time mid price for a symbol
    Price { symbol: String },
    /// View L2 order book
    Orderbook { symbol: String },
    /// View funding rate (current and historical)
    Funding { symbol: String },
    /// Place a buy/long order
    Buy {
        #[arg(long)]
        symbol: String,
        #[arg(long)]
        size: String,
        #[arg(long)]
        price: String,
        #[arg(long)]
        leverage: Option<u32>,
    },
    /// Place a sell/short order
    Sell {
        #[arg(long)]
        symbol: String,
        #[arg(long)]
        size: String,
        #[arg(long)]
        price: String,
    },
    /// Cancel an open order
    Cancel {
        #[arg(long)]
        symbol: String,
        #[arg(long)]
        order_id: u64,
    },
    /// View perpetual positions
    Positions,
    /// View account balances (USDC margin + spot)
    Balances,
    /// List open orders
    Orders {
        #[arg(long)]
        symbol: Option<String>,
    },
    /// Deposit USDC from Arbitrum to open/fund your Hyperliquid account
    Deposit {
        /// Amount in USDC (e.g. 10 for $10.00)
        #[arg(long)]
        amount: String,
    },
    /// Withdraw USDC from Hyperliquid back to an Arbitrum address
    Withdraw {
        /// Amount in USDC (e.g. 5 for $5.00)
        #[arg(long)]
        amount: String,
        /// Destination EVM address on Arbitrum (defaults to your onchainos wallet)
        #[arg(long)]
        destination: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli.command).await {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}

async fn run(cmd: Command) -> Result<()> {
    match cmd {
        Command::Markets => cmd_markets().await,
        Command::SpotMarkets => cmd_spot_markets().await,
        Command::Price { symbol } => cmd_price(&symbol).await,
        Command::Orderbook { symbol } => cmd_orderbook(&symbol).await,
        Command::Funding { symbol } => cmd_funding(&symbol).await,
        Command::Buy { symbol, size, price, leverage } => {
            cmd_buy(&symbol, &size, &price, leverage).await
        }
        Command::Sell { symbol, size, price } => cmd_sell(&symbol, &size, &price).await,
        Command::Cancel { symbol, order_id } => cmd_cancel(&symbol, order_id).await,
        Command::Positions => cmd_positions().await,
        Command::Balances => cmd_balances().await,
        Command::Orders { symbol } => cmd_orders(symbol).await,
        Command::Deposit { amount } => cmd_deposit(&amount).await,
        Command::Withdraw { amount, destination } => cmd_withdraw(&amount, destination.as_deref()).await,
    }
}

// ---------------------------------------------------------------------------
// Data commands (read-only)
// ---------------------------------------------------------------------------

async fn cmd_markets() -> Result<()> {
    let client = HyperliquidClient::new()?;

    let meta = client.info(json!({"type": "meta"})).await?;
    let mids = client.info(json!({"type": "allMids"})).await?;

    let universe = meta
        .get("universe")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mut markets: Vec<Value> = Vec::new();
    for asset in &universe {
        let symbol = asset.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let mid_price = mids.get(symbol).and_then(|v| v.as_str()).unwrap_or("0");
        markets.push(json!({
            "symbol": symbol,
            "mid_price": mid_price,
            "szDecimals": asset.get("szDecimals"),
            "maxLeverage": asset.get("maxLeverage"),
        }));
    }

    output(json!({ "markets": markets }));
    Ok(())
}

async fn cmd_spot_markets() -> Result<()> {
    let client = HyperliquidClient::new()?;
    let data = client.info(json!({"type": "spotMeta"})).await?;

    let universe = data
        .get("universe")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let tokens = data
        .get("tokens")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mut markets: Vec<Value> = Vec::new();
    for (i, pair) in universe.iter().enumerate() {
        let name = pair.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let base_idx = pair
            .get("tokens")
            .and_then(|v| v.get(0))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;
        let quote_idx = pair
            .get("tokens")
            .and_then(|v| v.get(1))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;
        let base = tokens
            .get(base_idx)
            .and_then(|t| t.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let quote = tokens
            .get(quote_idx)
            .and_then(|t| t.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        markets.push(json!({
            "name": name,
            "base": base,
            "quote": quote,
            "index": i,
        }));
    }

    output(json!({ "markets": markets }));
    Ok(())
}

async fn cmd_price(symbol: &str) -> Result<()> {
    let client = HyperliquidClient::new()?;
    let mids = client.info(json!({"type": "allMids"})).await?;

    let mid_price = mids
        .get(symbol)
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("symbol '{}' not found in allMids", symbol))?;

    output(json!({
        "symbol": symbol,
        "mid_price": mid_price,
    }));
    Ok(())
}

async fn cmd_orderbook(symbol: &str) -> Result<()> {
    let client = HyperliquidClient::new()?;
    let data = client
        .info(json!({"type": "l2Book", "coin": symbol}))
        .await?;
    output(data);
    Ok(())
}

async fn cmd_funding(symbol: &str) -> Result<()> {
    let client = HyperliquidClient::new()?;

    let meta = client.info(json!({"type": "meta"})).await?;
    let universe = meta
        .get("universe")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let current_funding = universe
        .iter()
        .find(|a| a.get("name").and_then(|v| v.as_str()) == Some(symbol))
        .and_then(|a| a.get("funding").cloned())
        .unwrap_or(Value::Null);

    let day_ago = chrono::Utc::now().timestamp_millis() - 86_400_000;
    let history = client
        .info(json!({
            "type": "fundingHistory",
            "coin": symbol,
            "startTime": day_ago,
        }))
        .await?;

    output(json!({
        "symbol": symbol,
        "current_funding": current_funding,
        "history_24h": history,
    }));
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn normalize_decimal(s: &str) -> String {
    if s.contains('.') {
        let trimmed = s.trim_end_matches('0').trim_end_matches('.');
        if trimmed.is_empty() {
            "0".to_string()
        } else {
            trimmed.to_string()
        }
    } else {
        s.to_string()
    }
}

async fn resolve_asset_index(client: &HyperliquidClient, symbol: &str) -> Result<u32> {
    let meta = client.info(json!({"type": "meta"})).await?;
    let universe = meta["universe"]
        .as_array()
        .context("failed to get universe from meta")?;
    for (i, asset) in universe.iter().enumerate() {
        if asset["name"].as_str() == Some(symbol) {
            return Ok(i as u32);
        }
    }
    bail!(
        "Symbol '{}' not found. Use 'dapp-hyperliquid markets' to see available symbols.",
        symbol
    )
}

// ---------------------------------------------------------------------------
// Trading commands
// ---------------------------------------------------------------------------

async fn cmd_buy(symbol: &str, size: &str, price: &str, leverage: Option<u32>) -> Result<()> {
    let client = HyperliquidClient::new_with_signer()?;
    let asset_index = resolve_asset_index(&client, symbol).await?;

    if let Some(lev) = leverage {
        let nonce = chrono::Utc::now().timestamp_millis() as u64;
        client
            .exchange(
                json!({
                    "type": "updateLeverage",
                    "asset": asset_index,
                    "isCross": true,
                    "leverage": lev,
                }),
                nonce,
                None,
            )
            .await?;
    }

    let nonce = chrono::Utc::now().timestamp_millis() as u64;
    let norm_price = normalize_decimal(price);
    let norm_size = normalize_decimal(size);
    let result = client
        .exchange(
            json!({
                "type": "order",
                "orders": [{
                    "a": asset_index,
                    "b": true,
                    "p": norm_price,
                    "s": norm_size,
                    "r": false,
                    "t": {"limit": {"tif": "Gtc"}}
                }],
                "grouping": "na"
            }),
            nonce,
            None,
        )
        .await?;

    if is_not_registered(&result) {
        bail!("Hyperliquid account not found — deposit USDC to open your account:\n  dapp-hyperliquid deposit --amount <USDC>");
    }
    output(json!({
        "action": "buy",
        "symbol": symbol,
        "size": size,
        "price": price,
        "leverage": leverage,
        "result": result,
    }));
    Ok(())
}

async fn cmd_sell(symbol: &str, size: &str, price: &str) -> Result<()> {
    let client = HyperliquidClient::new_with_signer()?;
    let asset_index = resolve_asset_index(&client, symbol).await?;

    let nonce = chrono::Utc::now().timestamp_millis() as u64;
    let norm_price = normalize_decimal(price);
    let norm_size = normalize_decimal(size);
    let result = client
        .exchange(
            json!({
                "type": "order",
                "orders": [{
                    "a": asset_index,
                    "b": false,
                    "p": norm_price,
                    "s": norm_size,
                    "r": false,
                    "t": {"limit": {"tif": "Gtc"}}
                }],
                "grouping": "na"
            }),
            nonce,
            None,
        )
        .await?;

    if is_not_registered(&result) {
        bail!("Hyperliquid account not found — deposit USDC to open your account:\n  dapp-hyperliquid deposit --amount <USDC>");
    }
    output(json!({
        "action": "sell",
        "symbol": symbol,
        "size": size,
        "price": price,
        "result": result,
    }));
    Ok(())
}

async fn cmd_cancel(symbol: &str, order_id: u64) -> Result<()> {
    let client = HyperliquidClient::new_with_signer()?;
    let asset_index = resolve_asset_index(&client, symbol).await?;

    let nonce = chrono::Utc::now().timestamp_millis() as u64;
    let result = client
        .exchange(
            json!({
                "type": "cancel",
                "cancels": [{
                    "a": asset_index,
                    "o": order_id
                }]
            }),
            nonce,
            None,
        )
        .await?;

    if is_not_registered(&result) {
        bail!("Hyperliquid account not found — deposit USDC to open your account:\n  dapp-hyperliquid deposit --amount <USDC>");
    }
    output(json!({
        "action": "cancel",
        "symbol": symbol,
        "order_id": order_id,
        "result": result,
    }));
    Ok(())
}

async fn cmd_positions() -> Result<()> {
    let client = HyperliquidClient::new_with_signer()?;
    let addr = client.address()?;
    let data = client
        .info(json!({
            "type": "clearinghouseState",
            "user": addr
        }))
        .await?;

    output(json!({
        "positions": &data["assetPositions"],
        "margin_summary": &data["marginSummary"],
        "cross_margin_summary": &data["crossMarginSummary"],
    }));
    Ok(())
}

async fn cmd_balances() -> Result<()> {
    let client = HyperliquidClient::new_with_signer()?;
    let addr = client.address()?;

    let perps = client
        .info(json!({
            "type": "clearinghouseState",
            "user": addr
        }))
        .await?;

    let spot = client
        .info(json!({
            "type": "spotClearinghouseState",
            "user": addr
        }))
        .await?;

    output(json!({
        "perps_margin": perps.get("marginSummary"),
        "spot_balances": spot.get("balances"),
    }));
    Ok(())
}

async fn cmd_orders(symbol: Option<String>) -> Result<()> {
    let client = HyperliquidClient::new_with_signer()?;
    let addr = client.address()?;
    let data = client
        .info(json!({
            "type": "openOrders",
            "user": addr
        }))
        .await?;

    let orders = if let Some(ref sym) = symbol {
        let filtered: Vec<&Value> = data
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter(|o| o["coin"].as_str() == Some(sym))
                    .collect()
            })
            .unwrap_or_default();
        json!(filtered)
    } else {
        data
    };

    output(json!({ "orders": orders }));
    Ok(())
}

// ---------------------------------------------------------------------------
// Withdraw
// ---------------------------------------------------------------------------

async fn cmd_withdraw(amount: &str, destination: Option<&str>) -> Result<()> {
    let usdc: f64 = amount.parse().context("invalid amount")?;
    if usdc < 2.0 {
        bail!("minimum withdrawal is $2 USDC");
    }

    let client = HyperliquidClient::new_with_signer()?;
    let hl_key = client.hl_key()?;

    // Default destination: AA wallet address
    let dest = match destination {
        Some(d) => d.to_string(),
        None => client.wallet_address()?,
    };

    let mainnet = auth::is_mainnet(client.base_url());
    let hl_chain = if mainnet { "Mainnet" } else { "Testnet" };
    let time = chrono::Utc::now().timestamp_millis() as u64;
    // Match Python str(float): 5.0 → "5.0", 5.5 → "5.5"
    let amount_str = if usdc.fract() == 0.0 {
        format!("{:.1}", usdc)
    } else {
        format!("{}", usdc)
    };

    let signature = auth::sign_withdraw(hl_key, hl_chain, &dest, &amount_str, time)?;

    let action = json!({
        "type": "withdraw3",
        "hyperliquidChain": hl_chain,
        "signatureChainId": "0x66eee",
        "destination": dest,
        "amount": amount_str,
        "time": time,
    });

    let result = client.post_exchange(json!({
        "action": action,
        "nonce": time,
        "signature": signature,
    })).await?;

    output(json!({
        "action": "withdraw",
        "amount_usdc": usdc,
        "destination": dest,
        "result": result,
    }));
    Ok(())
}

// ---------------------------------------------------------------------------
// Deposit (onboarding)
// ---------------------------------------------------------------------------

async fn cmd_deposit(amount: &str) -> Result<()> {
    let usdc: f64 = amount.parse().context("invalid amount — use a number like 10 or 50.5")?;
    if usdc < 5.0 {
        bail!("minimum deposit is $5 USDC — amounts below $5 are permanently lost");
    }
    // USDC has 6 decimals
    let raw_amount = (usdc * 1_000_000.0).round() as u64;

    let client = HyperliquidClient::new_with_signer()?;
    let wallet_addr = client.wallet_address()?; // AA wallet — source of USDC
    let hl_addr = client.address()?;            // local key address — Hyperliquid account + permit owner
    let hl_key = client.hl_key()?;

    let deadline = (chrono::Utc::now().timestamp() + 3600) as u64;

    // Step 1: transfer USDC from AA wallet to local key address (permit owner must == user)
    eprintln!("Transferring {usdc} USDC from {wallet_addr} → {hl_addr}...");
    onchainos_wallet_send(ARBITRUM_CHAIN_ID, &wallet_addr, &hl_addr, USDC_ARBITRUM, raw_amount)?;

    // Step 2: fetch USDC permit nonce for the local key address (now the USDC holder)
    eprintln!("Fetching USDC permit nonce...");
    let nonce = get_usdc_nonce(&hl_addr).await?;

    // Step 3: sign EIP-2612 permit locally (owner = hl_addr, spender = bridge)
    eprintln!("Signing USDC permit...");
    let (r, s, v) = crate::auth::sign_usdc_permit_local(
        hl_key, &hl_addr, HL_BRIDGE, raw_amount as u128, nonce, deadline,
    )?;

    // Step 4: call bridge — user = permit.owner = hl_addr
    eprintln!("Depositing to Hyperliquid (account: {hl_addr})...");
    let calldata = encode_batched_deposit_with_permit(&hl_addr, raw_amount, deadline, &r, &s, v)?;
    let deposit_tx = onchainos_contract_call(ARBITRUM_CHAIN_ID, HL_BRIDGE, &calldata, &wallet_addr)?;

    output(json!({
        "action": "deposit",
        "amount_usdc": usdc,
        "deposit_tx": deposit_tx,
        "note": "Account will be active on Hyperliquid within ~1 minute"
    }));
    Ok(())
}

// ---------------------------------------------------------------------------
// Deposit helpers (EIP-2612 permit flow)
// ---------------------------------------------------------------------------

/// Transfer an ERC-20 token via `onchainos wallet send`.
fn onchainos_wallet_send(chain_id: &str, from: &str, recipient: &str, token: &str, amount: u64) -> Result<()> {
    let out = std::process::Command::new("onchainos")
        .args([
            "wallet", "send",
            "--chain", chain_id,
            "--from", from,
            "--receipt", recipient,
            "--contract-token", token,
            "--amt", &amount.to_string(),
            "--force",
        ])
        .output()
        .context("onchainos not found")?;

    let stdout = String::from_utf8_lossy(&out.stdout);
    if !out.status.success() {
        if let Ok(resp) = serde_json::from_str::<Value>(&stdout) {
            if let Some(err) = resp["error"].as_str() {
                bail!("wallet send failed: {}", err);
            }
        }
        bail!("wallet send failed: {}", stdout.trim());
    }
    Ok(())
}

/// Fetch the current EIP-2612 nonce for `owner` from the USDC contract on Arbitrum.
async fn get_usdc_nonce(owner: &str) -> Result<u64> {
    use tiny_keccak::{Hasher, Keccak};
    // nonces(address) selector
    let mut k = Keccak::v256();
    let mut out = [0u8; 32];
    k.update(b"nonces(address)");
    k.finalize(&mut out);

    let addr_bytes = hex::decode(owner.strip_prefix("0x").unwrap_or(owner))
        .context("invalid owner address")?;
    let mut call_data = out[0..4].to_vec();
    let mut padded = [0u8; 32];
    padded[12..].copy_from_slice(&addr_bytes);
    call_data.extend_from_slice(&padded);

    let client = reqwest::Client::new();
    let resp = client
        .post(ARBITRUM_RPC)
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "eth_call",
            "params": [{"to": USDC_ARBITRUM, "data": format!("0x{}", hex::encode(&call_data))}, "latest"],
            "id": 1
        }))
        .send()
        .await
        .context("Arbitrum RPC call failed")?
        .json::<serde_json::Value>()
        .await
        .context("failed to parse Arbitrum RPC response")?;

    let hex_result = resp["result"]
        .as_str()
        .context("no result in eth_call response")?;
    let bytes = hex::decode(hex_result.strip_prefix("0x").unwrap_or(hex_result))
        .context("failed to decode nonce hex")?;
    if bytes.len() < 8 {
        bail!("unexpected nonce response length: {}", bytes.len());
    }
    let nonce = u64::from_be_bytes(bytes[bytes.len() - 8..].try_into().unwrap());
    Ok(nonce)
}

/// ABI-encode calldata for `batchedDepositWithPermit((address,uint64,uint64,(uint256,uint256,uint8))[])`.
fn encode_batched_deposit_with_permit(
    user: &str,
    usd: u64,
    deadline: u64,
    r: &[u8; 32],
    s: &[u8; 32],
    v: u8,
) -> Result<String> {
    use tiny_keccak::{Hasher, Keccak};
    let mut k = Keccak::v256();
    let mut out = [0u8; 32];
    k.update(b"batchedDepositWithPermit((address,uint64,uint64,(uint256,uint256,uint8))[])");
    k.finalize(&mut out);
    let selector = &out[0..4];

    let mut data: Vec<u8> = selector.to_vec();

    // offset to array data = 0x20
    let mut word = [0u8; 32];
    word[31] = 0x20;
    data.extend_from_slice(&word);

    // array length = 1
    let mut word = [0u8; 32];
    word[31] = 1;
    data.extend_from_slice(&word);

    // struct fields (all static — inline, no offsets):
    // address user (left-padded to 32 bytes)
    let addr_bytes = hex::decode(user.strip_prefix("0x").unwrap_or(user))
        .context("invalid user address")?;
    let mut padded = [0u8; 32];
    padded[12..].copy_from_slice(&addr_bytes);
    data.extend_from_slice(&padded);

    // uint64 usd (right-justified in 32 bytes)
    let mut padded = [0u8; 32];
    padded[24..].copy_from_slice(&usd.to_be_bytes());
    data.extend_from_slice(&padded);

    // uint64 deadline
    let mut padded = [0u8; 32];
    padded[24..].copy_from_slice(&deadline.to_be_bytes());
    data.extend_from_slice(&padded);

    // uint256 r
    data.extend_from_slice(r);

    // uint256 s
    data.extend_from_slice(s);

    // uint8 v (right-justified in 32 bytes)
    let mut padded = [0u8; 32];
    padded[31] = v;
    data.extend_from_slice(&padded);

    Ok(format!("0x{}", hex::encode(data)))
}

/// Call a contract via `onchainos wallet contract-call` and return tx hash.
fn onchainos_contract_call(chain_id: &str, to: &str, input_data: &str, from: &str) -> Result<String> {
    let out = std::process::Command::new("onchainos")
        .args([
            "wallet", "contract-call",
            "--chain", chain_id,
            "--to", to,
            "--input-data", input_data,
            "--from", from,
            "--gas-limit", "500000",
            "--force",
        ])
        .output()
        .context("onchainos not found")?;

    let stdout = String::from_utf8_lossy(&out.stdout);
    if !out.status.success() {
        if let Ok(resp) = serde_json::from_str::<Value>(&stdout) {
            if let Some(err) = resp["error"].as_str() {
                bail!("contract-call failed: {}", err);
            }
        }
        bail!("contract-call failed: {}", stdout.trim());
    }

    let resp: Value = serde_json::from_str(&stdout).context("failed to parse contract-call output")?;
    let tx = resp["data"]["txHash"]
        .as_str()
        .or_else(|| resp["data"]["tx_hash"].as_str())
        .unwrap_or("")
        .to_string();
    Ok(tx)
}

// ---------------------------------------------------------------------------
// Account existence check
// ---------------------------------------------------------------------------

/// Returns true when Hyperliquid responds with "does not exist" — account not onboarded.
fn is_not_registered(result: &Value) -> bool {
    result["status"].as_str() == Some("err")
        && result["response"]
            .as_str()
            .map(|s| s.contains("does not exist"))
            .unwrap_or(false)
}
