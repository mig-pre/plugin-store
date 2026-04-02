use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

/// Map of plugin name → download count.
pub type StatsMap = HashMap<String, u64>;

fn parse_stats(raw: HashMap<String, serde_json::Value>) -> StatsMap {
    raw.into_iter()
        .filter_map(|(k, v)| {
            let n = match &v {
                serde_json::Value::Number(n) => n.as_u64(),
                serde_json::Value::String(s) => s.parse().ok(),
                _ => None,
            };
            n.map(|n| (k, n))
        })
        .collect()
}

#[derive(Debug, Serialize, Deserialize)]
struct ReportPayload {
    name: String,
    version: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OkxReportPayload {
    plugin_name: String,
    div_id: String,
}

// HMAC secret for device ID signing
const HMAC_SECRET: &[u8] = b"plugin-store-okx-2026";

/// Generate a stable device ID from machine fingerprint + HMAC signature.
/// Format: 32-char device hash + 8-char HMAC sig = 40 chars total.
fn generate_device_token() -> String {
    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_default();
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    let home = dirs::home_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let raw = format!("{}:{}:{}:{}", hostname, os, arch, home);

    // Device ID: SHA-256 of fingerprint, first 32 hex chars
    let device_hash = hex::encode(Sha256::digest(raw.as_bytes()));
    let device_id = &device_hash[..32];

    // HMAC signature: SHA-256(secret + device_id), first 8 hex chars
    let mut hmac_hasher = Sha256::new();
    hmac_hasher.update(HMAC_SECRET);
    hmac_hasher.update(device_id.as_bytes());
    let sig = hex::encode(hmac_hasher.finalize());
    let sig_short = &sig[..8];

    format!("{}{}", device_id, sig_short)
}

/// Resolve stats base URL: registry value takes priority, fallback to env var.
fn resolve_url(registry_url: Option<&str>) -> Option<String> {
    registry_url
        .map(|s| s.to_string())
        .or_else(|| std::env::var("PLUGIN_STORE_STATS_URL").ok())
}

/// Fetch download counts from the stats API.
/// GET {stats_url}/counts → {"plugin-name": 123, ...}
/// Returns an empty map on any error or if the URL is not configured.
pub async fn fetch(registry_url: Option<&str>) -> StatsMap {
    let Some(base) = resolve_url(registry_url) else {
        return HashMap::new();
    };
    let url = format!("{}/counts", base.trim_end_matches('/'));
    let Ok(resp) = reqwest::Client::new()
        .get(&url)
        .header("User-Agent", "plugin-store")
        .send()
        .await
    else {
        return HashMap::new();
    };
    let raw: HashMap<String, serde_json::Value> = resp.json().await.unwrap_or_default();
    parse_stats(raw)
}

/// Report a successful install (fire-and-forget, dual endpoint).
/// 1. POST {stats_url}/install → Vercel stats (existing)
/// 2. POST okx API → OKX download report (new)
pub async fn report_install(name: &str, version: &str, registry_url: Option<&str>) {
    let client = reqwest::Client::new();
    let device_token = generate_device_token();

    // ── Vercel stats (existing) ──
    if let Some(base) = resolve_url(registry_url) {
        let url = format!("{}/install", base.trim_end_matches('/'));
        let payload = ReportPayload {
            name: name.to_string(),
            version: version.to_string(),
        };
        let _ = client
            .post(&url)
            .header("User-Agent", "plugin-store")
            .json(&payload)
            .send()
            .await;
    }

    // ── OKX download report (new) ──
    let okx_payload = OkxReportPayload {
        plugin_name: name.to_string(),
        div_id: device_token,
    };
    let _ = client
        .post("https://www.okx.com/priapi/v1/wallet/plugins/download/report")
        .header("User-Agent", "plugin-store")
        .json(&okx_payload)
        .send()
        .await;
}
