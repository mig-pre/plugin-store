/// Wrappers around the `onchainos` CLI for wallet resolution + EVM contract calls.
/// The plugin holds NO private keys.
///
/// Knowledge-base compliance:
///   - ONC-001: --force is defensively included (skips backend risk-control prompts).
///     Low-risk daily calls work without it; rare risk-control paths (unlimited approve,
///     untrusted contracts) need it to avoid silent revert. Always-pass is a no-op
///     in the common case.
///   - EVM-015: explicit gas_limit override. Dolomite operate(...) actions are
///     ABI-heavy + multi-storage-write — onchainos auto-estimate can under-shoot.

use std::process::Command;
use serde_json::Value;

pub fn resolve_wallet(chain_id: u64) -> anyhow::Result<String> {
    let output = Command::new("onchainos")
        .args(["wallet", "addresses"])
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to spawn onchainos: {} (is it on PATH?)", e))?;
    if !output.status.success() {
        anyhow::bail!(
            "onchainos wallet addresses failed: stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout).trim(),
            String::from_utf8_lossy(&output.stderr).trim(),
        );
    }
    let json: Value = serde_json::from_str(&String::from_utf8_lossy(&output.stdout))
        .map_err(|e| anyhow::anyhow!("parse onchainos JSON failed: {}", e))?;
    let chain_id_str = chain_id.to_string();
    if let Some(evm_list) = json["data"]["evm"].as_array() {
        for entry in evm_list {
            if entry["chainIndex"].as_str() == Some(&chain_id_str) {
                if let Some(addr) = entry["address"].as_str() {
                    return Ok(addr.to_string());
                }
            }
        }
        if let Some(first) = evm_list.first() {
            if let Some(addr) = first["address"].as_str() {
                return Ok(addr.to_string());
            }
        }
    }
    anyhow::bail!("Could not resolve wallet address for chain {}", chain_id)
}

pub fn wallet_contract_call(
    chain_id: u64,
    to: &str,
    calldata: &str,
    value_wei: Option<u128>,
    gas_limit: Option<u64>,
    dry_run: bool,
) -> anyhow::Result<Value> {
    if dry_run {
        return Ok(serde_json::json!({
            "ok": true,
            "dry_run": true,
            "chain": chain_id,
            "to": to,
            "data": calldata,
            "value_wei": value_wei.map(|v| v.to_string()),
            "gas_limit": gas_limit.map(|g| g.to_string()),
            "note": "Dry run — calldata not submitted"
        }));
    }
    let mut args = vec![
        "wallet".to_string(),
        "contract-call".to_string(),
        "--force".to_string(),  // ← ONC-001 defensive
        "--chain".to_string(),
        chain_id.to_string(),
        "--to".to_string(),
        to.to_string(),
        "--input-data".to_string(),
        calldata.to_string(),
    ];
    if let Some(v) = value_wei {
        args.push("--amt".to_string());
        args.push(v.to_string());
    }
    if let Some(g) = gas_limit {
        args.push("--gas-limit".to_string());
        args.push(g.to_string());
    }
    let output = Command::new("onchainos").args(&args).output()
        .map_err(|e| anyhow::anyhow!("Failed to spawn onchainos: {}", e))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let detail = if stdout.trim().is_empty() {
            stderr.to_string()
        } else {
            stdout.to_string()
        };
        anyhow::bail!("onchainos contract-call failed: {}", detail.trim());
    }
    let result: Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|_| serde_json::json!({ "raw": stdout.to_string() }));
    Ok(result)
}

pub fn extract_tx_hash(result: &Value) -> Option<String> {
    for path in [("data", "txHash"), ("data", "hash")] {
        if let Some(s) = result[path.0][path.1].as_str() {
            return Some(s.to_string());
        }
    }
    if let Some(s) = result["txHash"].as_str() { return Some(s.to_string()); }
    if let Some(s) = result["hash"].as_str()    { return Some(s.to_string()); }
    None
}
