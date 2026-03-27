use anyhow::{bail, Context, Result};
use reqwest::Client;
use serde_json::{json, Value};

const BASE_URL: &str = "https://api.hyperliquid.xyz";

pub struct HyperliquidClient {
    http: Client,
    base_url: String,
    /// onchainos AA wallet address — holds USDC on Arbitrum, used as permit owner
    wallet_address: Option<String>,
    /// Local Hyperliquid trading key — signs all exchange actions
    hl_key: Option<k256::ecdsa::SigningKey>,
}

impl HyperliquidClient {
    pub fn new() -> Result<Self> {
        let base_url = std::env::var("HYPERLIQUID_URL").unwrap_or_else(|_| BASE_URL.to_string());
        Ok(Self {
            http: Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .build()?,
            base_url,
            wallet_address: None,
            hl_key: None,
        })
    }

    /// Create a client with signing capabilities.
    /// Loads/generates a local Hyperliquid key and resolves the AA wallet address.
    pub fn new_with_signer() -> Result<Self> {
        let base_url = std::env::var("HYPERLIQUID_URL").unwrap_or_else(|_| BASE_URL.to_string());
        let wallet_address = get_onchainos_evm_address()?;
        let hl_key = get_or_create_hl_key()?;
        Ok(Self {
            http: Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .build()?,
            base_url,
            wallet_address: Some(wallet_address),
            hl_key: Some(hl_key),
        })
    }

    /// Returns the Hyperliquid trading address (derived from the local key).
    pub fn address(&self) -> Result<String> {
        self.hl_key
            .as_ref()
            .map(hl_key_address)
            .context("Hyperliquid key not available")
    }

    /// Returns a reference to the local k256 signing key.
    pub fn hl_key(&self) -> Result<&k256::ecdsa::SigningKey> {
        self.hl_key.as_ref().context("Hyperliquid key not available")
    }

    /// Returns the onchainos AA wallet address (for USDC permit owner).
    pub fn wallet_address(&self) -> Result<String> {
        self.wallet_address
            .clone()
            .context("onchainos wallet not available — please login first")
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Post a pre-signed body directly to /exchange (used for non-trading actions like withdraw).
    pub async fn post_exchange(&self, body: Value) -> Result<Value> {
        let url = format!("{}/exchange", self.base_url.trim_end_matches('/'));
        let resp = self
            .http
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Hyperliquid /exchange request failed")?;
        self.handle_response(resp).await
    }

    pub async fn info(&self, body: Value) -> Result<Value> {
        let url = format!("{}/info", self.base_url.trim_end_matches('/'));
        let resp = self
            .http
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Hyperliquid /info request failed")?;
        self.handle_response(resp).await
    }

    pub async fn exchange(
        &self,
        action: Value,
        nonce: u64,
        vault_address: Option<&str>,
    ) -> Result<Value> {
        let key = self
            .hl_key
            .as_ref()
            .context("Hyperliquid key not available")?;

        let mainnet = crate::auth::is_mainnet(&self.base_url);
        let signature = crate::auth::sign_action(key, &action, nonce, vault_address, mainnet)?;

        let body = json!({
            "action": action,
            "nonce": nonce,
            "signature": signature,
            "vaultAddress": vault_address,
        });

        let url = format!("{}/exchange", self.base_url.trim_end_matches('/'));
        let resp = self
            .http
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Hyperliquid /exchange request failed")?;
        self.handle_response(resp).await
    }

    async fn handle_response(&self, resp: reqwest::Response) -> Result<Value> {
        let status = resp.status();
        if status.as_u16() == 429 {
            bail!("Rate limited — retry with backoff");
        }
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            bail!("Hyperliquid API error (HTTP {}): {}", status.as_u16(), body);
        }
        let body: Value = resp
            .json()
            .await
            .context("failed to parse Hyperliquid response")?;
        Ok(body)
    }
}

// ---------------------------------------------------------------------------
// Key management
// ---------------------------------------------------------------------------

/// Return the path for the local Hyperliquid trading key.
fn hl_key_path() -> Result<std::path::PathBuf> {
    let home = std::env::var("HOME").context("HOME env var not set")?;
    Ok(std::path::PathBuf::from(home).join(".config/dapp-hyperliquid/key.hex"))
}

/// Load existing key or generate and persist a new one.
pub fn get_or_create_hl_key() -> Result<k256::ecdsa::SigningKey> {
    // Allow override via environment variable
    if let Ok(hex_str) = std::env::var("HL_TRADING_KEY") {
        let bytes = hex::decode(hex_str.trim()).context("invalid HL_TRADING_KEY")?;
        return k256::ecdsa::SigningKey::from_slice(&bytes).context("invalid HL_TRADING_KEY");
    }

    let key_path = hl_key_path()?;

    if key_path.exists() {
        let hex_str =
            std::fs::read_to_string(&key_path).context("failed to read Hyperliquid key file")?;
        let bytes = hex::decode(hex_str.trim()).context("invalid key in key file")?;
        return k256::ecdsa::SigningKey::from_slice(&bytes).context("invalid key in key file");
    }

    // Generate new key from OS entropy
    let key = generate_hl_key()?;
    let key_dir = key_path.parent().unwrap();
    std::fs::create_dir_all(key_dir).context("failed to create key directory")?;
    std::fs::write(&key_path, hex::encode(key.to_bytes()))
        .context("failed to write Hyperliquid key")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&key_path, std::fs::Permissions::from_mode(0o600));
    }

    let addr = hl_key_address(&key);
    eprintln!(
        "Generated new Hyperliquid trading key → {}\n  Key saved to: {}",
        addr,
        key_path.display()
    );
    Ok(key)
}

fn generate_hl_key() -> Result<k256::ecdsa::SigningKey> {
    use std::io::Read;
    let mut bytes = [0u8; 32];
    let mut f = std::fs::File::open("/dev/urandom").context("failed to open /dev/urandom")?;
    f.read_exact(&mut bytes).context("failed to read random bytes")?;
    k256::ecdsa::SigningKey::from_slice(&bytes).context("key generation failed")
}

/// Derive the EVM address from a k256 signing key.
pub fn hl_key_address(key: &k256::ecdsa::SigningKey) -> String {
    use k256::elliptic_curve::sec1::EncodedPoint;
    use tiny_keccak::{Hasher, Keccak};

    let point: EncodedPoint<k256::Secp256k1> = key.verifying_key().to_encoded_point(false);
    let pub_bytes = &point.as_bytes()[1..]; // skip 0x04 uncompressed prefix

    let mut keccak = Keccak::v256();
    let mut hash = [0u8; 32];
    keccak.update(pub_bytes);
    keccak.finalize(&mut hash);

    format!("0x{}", hex::encode(&hash[12..]))
}

// ---------------------------------------------------------------------------
// onchainos wallet
// ---------------------------------------------------------------------------

/// Resolve the AA wallet address from the currently logged-in onchainos account.
pub fn get_onchainos_evm_address() -> Result<String> {
    let output = std::process::Command::new("onchainos")
        .args(["wallet", "addresses"])
        .output()
        .context("onchainos not found — please install onchainos")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        if let Ok(resp) = serde_json::from_str::<Value>(&stdout) {
            if let Some(err) = resp["error"].as_str() {
                bail!("onchainos wallet not available — please login first ({})", err);
            }
        }
        bail!("onchainos wallet not available — please login first");
    }

    let resp: Value =
        serde_json::from_str(&stdout).context("failed to parse onchainos addresses output")?;

    let addr = resp["data"]["evm"]
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|entry| entry["address"].as_str())
        .context("onchainos wallet not available — please login first")?;

    Ok(addr.to_string())
}
