//! EIP-712 signing for Hyperliquid exchange endpoint.
//!
//! Hyperliquid uses a "phantom agent" signing scheme with chainId=1337.
//! Because onchainos does not support custom chain IDs, we sign locally
//! using a dedicated Hyperliquid trading key stored at
//! `~/.config/dapp-hyperliquid/key.hex`.
//!
//! Signing flow:
//! 1. Msgpack-encode action + nonce + vault flag → keccak256 → `connectionId`
//! 2. Build EIP-712 domain (name="Exchange", version="1", chainId=1337, verifyingContract=0x0)
//! 3. Build Agent struct { source="a"/"b", connectionId }
//! 4. Compute digest = keccak256("\x19\x01" + domainSeparator + structHash)
//! 5. Sign digest with local k256 signing key → { r, s, v }

use anyhow::{Context, Result};

use serde_json::Value;
use tiny_keccak::{Hasher, Keccak};

/// USDC on Arbitrum One — verifyingContract for EIP-2612 permit.
const USDC_ARBITRUM: &str = "af88d065e77c8cC2239327C5EDb3A432268e5831";

fn keccak(data: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak::v256();
    let mut out = [0u8; 32];
    hasher.update(data);
    hasher.finalize(&mut out);
    out
}

fn action_hash(action: &Value, nonce: u64, vault_address: Option<&str>) -> Result<[u8; 32]> {
    let packed = rmp_serde::to_vec_named(action).context("msgpack encode failed")?;
    let mut data = packed;
    data.extend_from_slice(&nonce.to_be_bytes());
    match vault_address {
        None => data.push(0x00),
        Some(addr) => {
            data.push(0x01);
            let addr_bytes =
                hex::decode(addr.strip_prefix("0x").unwrap_or(addr)).context("invalid vault address")?;
            data.extend_from_slice(&addr_bytes);
        }
    }
    Ok(keccak(&data))
}

pub fn is_mainnet(base_url: &str) -> bool {
    !base_url.contains("testnet")
}

/// Compute the full EIP-712 digest for a Hyperliquid exchange action.
fn compute_hl_eip712_digest(
    action: &Value,
    nonce: u64,
    vault_address: Option<&str>,
    mainnet: bool,
) -> Result<[u8; 32]> {
    let conn_id = action_hash(action, nonce, vault_address)?;
    let source = if mainnet { "a" } else { "b" };

    // Agent struct hash
    let agent_typehash = keccak(b"Agent(string source,bytes32 connectionId)");
    let source_hash = keccak(source.as_bytes());
    let mut struct_buf = [0u8; 96];
    struct_buf[..32].copy_from_slice(&agent_typehash);
    struct_buf[32..64].copy_from_slice(&source_hash);
    struct_buf[64..96].copy_from_slice(&conn_id);
    let struct_hash = keccak(&struct_buf);

    // Domain separator (chainId=1337, verifyingContract=0x0000...0000)
    let domain_sep = exchange_domain_sep();

    // Final EIP-712 digest: keccak256("\x19\x01" || domainSep || structHash)
    let mut final_buf = [0u8; 66];
    final_buf[0] = 0x19;
    final_buf[1] = 0x01;
    final_buf[2..34].copy_from_slice(&domain_sep);
    final_buf[34..66].copy_from_slice(&struct_hash);
    Ok(keccak(&final_buf))
}

/// Sign a USDC EIP-2612 permit locally.
/// Domain: name="USD Coin", version="2", chainId=42161, verifyingContract=USDC on Arbitrum.
/// Returns (r, s, v) where v uses Ethereum convention (27 or 28).
pub fn sign_usdc_permit_local(
    key: &k256::ecdsa::SigningKey,
    owner: &str,
    spender: &str,
    value: u128,
    nonce: u64,
    deadline: u64,
) -> Result<([u8; 32], [u8; 32], u8)> {
    let usdc_bytes = hex::decode(USDC_ARBITRUM).expect("hardcoded USDC address");

    // Domain separator: EIP712Domain(name, version, chainId, verifyingContract)
    let domain_typehash = keccak(
        b"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)",
    );
    let name_hash = keccak(b"USD Coin");
    let version_hash = keccak(b"2");
    let mut domain_buf = [0u8; 160]; // 5 × 32 bytes
    domain_buf[..32].copy_from_slice(&domain_typehash);
    domain_buf[32..64].copy_from_slice(&name_hash);
    domain_buf[64..96].copy_from_slice(&version_hash);
    // chainId = 42161 as uint256 (last 8 bytes of 32-byte slot at offset 96)
    domain_buf[120..128].copy_from_slice(&42161u64.to_be_bytes());
    // verifyingContract = USDC address (last 20 bytes of 32-byte slot at offset 128)
    domain_buf[140..160].copy_from_slice(&usdc_bytes);
    let domain_sep = keccak(&domain_buf);

    // Permit struct hash
    let permit_typehash = keccak(
        b"Permit(address owner,address spender,uint256 value,uint256 nonce,uint256 deadline)",
    );
    let owner_bytes =
        hex::decode(owner.strip_prefix("0x").unwrap_or(owner)).context("invalid owner address")?;
    let spender_bytes = hex::decode(spender.strip_prefix("0x").unwrap_or(spender))
        .context("invalid spender address")?;
    let mut struct_buf = [0u8; 192]; // 6 × 32 bytes
    struct_buf[..32].copy_from_slice(&permit_typehash);
    // address owner — 20 bytes, right-justified in slot 1 (32..64)
    struct_buf[44..64].copy_from_slice(&owner_bytes);
    // address spender — right-justified in slot 2 (64..96)
    struct_buf[76..96].copy_from_slice(&spender_bytes);
    // uint256 value — from u128 (16 bytes), right-justified in slot 3 (96..128)
    struct_buf[112..128].copy_from_slice(&value.to_be_bytes());
    // uint256 nonce — from u64 (8 bytes), right-justified in slot 4 (128..160)
    struct_buf[152..160].copy_from_slice(&nonce.to_be_bytes());
    // uint256 deadline — from u64 (8 bytes), right-justified in slot 5 (160..192)
    struct_buf[184..192].copy_from_slice(&deadline.to_be_bytes());
    let struct_hash = keccak(&struct_buf);

    // Final EIP-712 digest
    let mut final_buf = [0u8; 66];
    final_buf[0] = 0x19;
    final_buf[1] = 0x01;
    final_buf[2..34].copy_from_slice(&domain_sep);
    final_buf[34..66].copy_from_slice(&struct_hash);
    let digest = keccak(&final_buf);

    let (sig, rec_id): (k256::ecdsa::Signature, k256::ecdsa::RecoveryId) =
        key.sign_prehash_recoverable(&digest).context("permit signing failed")?;
    let sig_bytes = sig.to_bytes();
    let v = u8::from(rec_id) + 27u8;
    let r: [u8; 32] = sig_bytes[..32].try_into().unwrap();
    let s: [u8; 32] = sig_bytes[32..].try_into().unwrap();
    Ok((r, s, v))
}

/// Sign a Hyperliquid `withdraw3` user-signed action.
/// Domain: name="HyperliquidSignTransaction", version="1", chainId=0x66eee (421110).
/// Fields signed: hyperliquidChain, destination, amount, time.
pub fn sign_withdraw(
    key: &k256::ecdsa::SigningKey,
    hl_chain: &str,    // "Mainnet" or "Testnet"
    destination: &str,
    amount: &str,      // decimal string, e.g. "5.0"
    time: u64,         // unix ms
) -> Result<Value> {
    // Domain separator for user-signed actions
    let domain_typehash = keccak(
        b"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)",
    );
    let name_hash = keccak(b"HyperliquidSignTransaction");
    let version_hash = keccak(b"1");
    let mut domain_buf = [0u8; 160];
    domain_buf[..32].copy_from_slice(&domain_typehash);
    domain_buf[32..64].copy_from_slice(&name_hash);
    domain_buf[64..96].copy_from_slice(&version_hash);
    // chainId = 0x66eee = 421614
    domain_buf[120..128].copy_from_slice(&421614u64.to_be_bytes());
    let domain_sep = keccak(&domain_buf);

    // Struct hash: HyperliquidTransaction:Withdraw(string,string,string,uint64)
    let typehash = keccak(
        b"HyperliquidTransaction:Withdraw(string hyperliquidChain,string destination,string amount,uint64 time)",
    );
    let chain_hash = keccak(hl_chain.as_bytes());
    let dest_hash = keccak(destination.as_bytes());
    let amount_hash = keccak(amount.as_bytes());

    let mut struct_buf = [0u8; 160]; // 5 × 32
    struct_buf[..32].copy_from_slice(&typehash);
    struct_buf[32..64].copy_from_slice(&chain_hash);
    struct_buf[64..96].copy_from_slice(&dest_hash);
    struct_buf[96..128].copy_from_slice(&amount_hash);
    // uint64 time — right-justified in 32-byte slot
    struct_buf[152..160].copy_from_slice(&time.to_be_bytes());
    let struct_hash = keccak(&struct_buf);

    let mut final_buf = [0u8; 66];
    final_buf[0] = 0x19;
    final_buf[1] = 0x01;
    final_buf[2..34].copy_from_slice(&domain_sep);
    final_buf[34..66].copy_from_slice(&struct_hash);
    let digest = keccak(&final_buf);


    let (sig, rec_id): (k256::ecdsa::Signature, k256::ecdsa::RecoveryId) = key
        .sign_prehash_recoverable(&digest)
        .context("withdraw signing failed")?;
    let sig_bytes = sig.to_bytes();
    let v = u8::from(rec_id) + 27u8;
    Ok(serde_json::json!({
        "r": format!("0x{}", hex::encode(&sig_bytes[..32])),
        "s": format!("0x{}", hex::encode(&sig_bytes[32..])),
        "v": v,
    }))
}

fn exchange_domain_sep() -> [u8; 32] {
    let domain_typehash = keccak(
        b"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)",
    );
    let name_hash = keccak(b"Exchange");
    let version_hash = keccak(b"1");
    let mut domain_buf = [0u8; 160];
    domain_buf[..32].copy_from_slice(&domain_typehash);
    domain_buf[32..64].copy_from_slice(&name_hash);
    domain_buf[64..96].copy_from_slice(&version_hash);
    domain_buf[120..128].copy_from_slice(&1337u64.to_be_bytes());
    keccak(&domain_buf)
}

/// Sign a Hyperliquid action with a local k256 signing key.
pub fn sign_action(
    key: &k256::ecdsa::SigningKey,
    action: &Value,
    nonce: u64,
    vault_address: Option<&str>,
    mainnet: bool,
) -> Result<Value> {
    let digest = compute_hl_eip712_digest(action, nonce, vault_address, mainnet)?;

    let (sig, rec_id): (k256::ecdsa::Signature, k256::ecdsa::RecoveryId) = key
        .sign_prehash_recoverable(&digest)
        .context("EIP-712 signing failed")?;

    let sig_bytes = sig.to_bytes();
    let v = u8::from(rec_id) + 27u8; // Ethereum convention: 27 or 28

    Ok(serde_json::json!({
        "r": format!("0x{}", hex::encode(&sig_bytes[..32])),
        "s": format!("0x{}", hex::encode(&sig_bytes[32..])),
        "v": v,
    }))
}
