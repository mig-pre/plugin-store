use anyhow::Context;

pub const CHAIN_ETH: u64 = 1;
pub const CHAIN_ARB: u64 = 42161;

pub fn rpc_url(chain_id: u64) -> anyhow::Result<&'static str> {
    match chain_id {
        1     => Ok("https://ethereum.publicnode.com"),
        42161 => Ok("https://arb1.arbitrum.io/rpc"),
        _     => anyhow::bail!("Unsupported chain ID {}. Use 1 (Ethereum) or 42161 (Arbitrum).", chain_id),
    }
}

pub fn chain_name(chain_id: u64) -> &'static str {
    match chain_id {
        1     => "Ethereum",
        42161 => "Arbitrum",
        _     => "Unknown",
    }
}

/// Make a raw eth_call and return the result hex string (no 0x prefix).
pub async fn eth_call(chain_id: u64, to: &str, data: &str) -> anyhow::Result<String> {
    let rpc = rpc_url(chain_id)?;
    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_call",
        "id": 1,
        "params": [{"to": to, "data": data}, "latest"]
    });
    let resp: serde_json::Value = client
        .post(rpc)
        .json(&body)
        .send().await
        .context("RPC request failed")?
        .json().await
        .context("Failed to parse RPC response")?;

    if let Some(err) = resp.get("error") {
        anyhow::bail!("RPC error: {}", err);
    }
    let result = resp.get("result")
        .and_then(|r| r.as_str())
        .ok_or_else(|| anyhow::anyhow!("No result in RPC response"))?;
    Ok(result.trim_start_matches("0x").to_string())
}
