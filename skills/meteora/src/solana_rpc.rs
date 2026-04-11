use base64::{Engine as _, engine::general_purpose::STANDARD as B64};
use reqwest::Client;
use serde_json::{json, Value};

const SOLANA_RPC: &str = "https://api.mainnet-beta.solana.com";
const SOLANA_RPC_FALLBACK: &str = "https://rpc.ankr.com/solana";

/// POST to Solana RPC; falls back to secondary endpoint on 429.
async fn rpc_call(client: &Client, body: &Value) -> anyhow::Result<Value> {
    let endpoints = [SOLANA_RPC, SOLANA_RPC_FALLBACK];
    for (i, endpoint) in endpoints.iter().enumerate() {
        if i > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        }
        for attempt in 0u32..2 {
            if attempt > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(800)).await;
            }
            let resp = client.post(*endpoint).json(body).send().await?;
            if resp.status().as_u16() == 429 {
                continue;
            }
            return resp.json::<Value>().await.map_err(|e| anyhow::anyhow!("RPC JSON parse: {e}"));
        }
    }
    anyhow::bail!("Solana RPC rate limited on all endpoints")
}

pub async fn get_account_data(client: &Client, address: &str) -> anyhow::Result<Vec<u8>> {
    let body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getAccountInfo",
        "params": [address, {"encoding": "base64"}]
    });
    let resp: Value = rpc_call(client, &body).await?;

    let data_b64 = resp["result"]["value"]["data"][0]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Account not found: {address}"))?;

    Ok(B64.decode(data_b64)?)
}

/// Find the user's token account for a given mint.
/// Prefers the ATA if it exists; falls back to any token account via getTokenAccountsByOwner.
/// Returns (account_pubkey, exists).
pub async fn find_token_account(
    client: &Client,
    wallet: &str,
    mint: &str,
    ata: &str, // precomputed ATA address to try first
) -> anyhow::Result<(String, bool)> {
    // Fast path: check if ATA exists
    if account_exists(client, ata).await? {
        return Ok((ata.to_string(), true));
    }

    // Slow path: find any token account for this mint
    let body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getTokenAccountsByOwner",
        "params": [
            wallet,
            {"mint": mint},
            {"encoding": "jsonParsed"}
        ]
    });
    let resp: Value = rpc_call(client, &body).await?;

    if let Some(acc) = resp["result"]["value"].as_array().and_then(|a| a.first()) {
        let pubkey = acc["pubkey"].as_str().unwrap_or("").to_string();
        if !pubkey.is_empty() {
            return Ok((pubkey, true));
        }
    }

    Ok((ata.to_string(), false))
}

pub async fn account_exists(client: &Client, address: &str) -> anyhow::Result<bool> {
    let body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getAccountInfo",
        "params": [address, {"encoding": "base64"}]
    });
    let resp: Value = rpc_call(client, &body).await?;
    Ok(resp["result"]["value"].is_object())
}

pub async fn get_latest_blockhash(client: &Client) -> anyhow::Result<[u8; 32]> {
    let body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getLatestBlockhash",
        "params": [{"commitment": "confirmed"}]
    });
    let resp: Value = rpc_call(client, &body).await?;

    let hash_str = resp["result"]["value"]["blockhash"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No blockhash in RPC response"))?;

    let bytes = bs58::decode(hash_str).into_vec()?;
    anyhow::ensure!(bytes.len() == 32, "Invalid blockhash length: {}", bytes.len());
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    Ok(arr)
}

/// Parsed fields from an LbPair account
pub struct LbPairInfo {
    pub active_id: i32,
    pub bin_step: u16,
    pub token_x_mint: [u8; 32],
    pub token_y_mint: [u8; 32],
    pub reserve_x: [u8; 32],
    pub reserve_y: [u8; 32],
}

/// Parse an LbPair account buffer.
///
/// Offsets verified against Meteora DLMM IDL struct layout:
///   8  anchor discriminator
///   +68 fields before active_id  → offset 76
///   active_id   i32  [76..80]
///   bin_step    u16  [80..82]
///   6 bytes pad  →  offset 88
///   token_x_mint Pubkey [88..120]
///   token_y_mint Pubkey [120..152]
///   reserve_x    Pubkey [152..184]
///   reserve_y    Pubkey [184..216]
pub fn parse_lb_pair(data: &[u8]) -> anyhow::Result<LbPairInfo> {
    anyhow::ensure!(
        data.len() >= 216,
        "LbPair account data too short: {} bytes (expected ≥216)",
        data.len()
    );

    let active_id = i32::from_le_bytes(data[76..80].try_into()?);
    let bin_step = u16::from_le_bytes(data[80..82].try_into()?);

    let mut token_x_mint = [0u8; 32];
    token_x_mint.copy_from_slice(&data[88..120]);
    let mut token_y_mint = [0u8; 32];
    token_y_mint.copy_from_slice(&data[120..152]);
    let mut reserve_x = [0u8; 32];
    reserve_x.copy_from_slice(&data[152..184]);
    let mut reserve_y = [0u8; 32];
    reserve_y.copy_from_slice(&data[184..216]);

    Ok(LbPairInfo {
        active_id,
        bin_step,
        token_x_mint,
        token_y_mint,
        reserve_x,
        reserve_y,
    })
}

/// Get token decimals from SPL Mint account data.
/// Decimals live at offset 44 in the standard Mint layout.
pub fn parse_mint_decimals(data: &[u8]) -> u8 {
    if data.len() > 44 {
        data[44]
    } else {
        6
    }
}
