use anyhow::Context;
use std::process::Command;

pub fn resolve_wallet(chain_id: u64) -> anyhow::Result<String> {
    let output = Command::new("onchainos")
        .args(["wallet", "addresses", "--chain", &chain_id.to_string(), "--output", "json"])
        .output()
        .context("onchainos not found — install from https://docs.onchainos.com")?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let v: serde_json::Value = serde_json::from_str(stdout.trim())
        .context("Could not parse onchainos wallet output")?;
    let addr = v.get(0)
        .and_then(|a| a.get("address"))
        .and_then(|a| a.as_str())
        .ok_or_else(|| anyhow::anyhow!(
            "No EVM wallet found for chain {}. Run: onchainos wallet login", chain_id
        ))?;
    Ok(addr.to_string())
}

pub fn wallet_contract_call(
    chain_id: u64,
    to: &str,
    data: &str,
    value: &str,
    wait: bool,
    dry_run: bool,
    from: Option<&str>,
) -> anyhow::Result<String> {
    let mut args = vec![
        "wallet".to_string(),
        "contract-call".to_string(),
        "--chain".to_string(), chain_id.to_string(),
        "--to".to_string(), to.to_string(),
        "--data".to_string(), data.to_string(),
        "--value".to_string(), value.to_string(),
        "--output".to_string(), "json".to_string(),
    ];
    if wait { args.push("--wait".to_string()); }
    if dry_run { args.push("--dry-run".to_string()); }
    if let Some(f) = from { args.extend(["--from".to_string(), f.to_string()]); }

    let output = Command::new("onchainos").args(&args).output()
        .context("onchainos not found")?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    if stdout.trim().is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        anyhow::bail!("onchainos error: {}", stderr.trim());
    }
    Ok(stdout)
}

pub fn extract_tx_hash(output: &str) -> String {
    serde_json::from_str::<serde_json::Value>(output.trim())
        .ok()
        .and_then(|v| {
            v.get("txHash").or_else(|| v.get("tx_hash"))
                .and_then(|h| h.as_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "0x0000000000000000000000000000000000000000000000000000000000000000".to_string())
}
