use crate::abi::{selector, calldata, calldata_raw, encode_address_array, encode_address, word, word_to_address, word_to_bool};
use crate::chain::eth_call;
use crate::contracts::*;

#[derive(Debug, Clone)]
pub struct VaultInfo {
    pub address: String,
    pub is_smart_col: bool,
    pub is_smart_debt: bool,
    pub col_token: String,
    pub debt_token: String,
}

/// Fetch all vault addresses from the resolver.
pub async fn all_vault_addresses(chain_id: u64) -> anyhow::Result<Vec<String>> {
    let sel = selector("getAllVaultsAddresses()");
    let data = calldata(sel, &[]);
    let result = eth_call(chain_id, VAULT_RESOLVER, &data).await?;

    // ABI: (address[]) = offset(32) + length + elements
    if result.len() < 128 {
        return Ok(vec![]);
    }
    let length = usize::from_str_radix(&result[64..128], 16).unwrap_or(0);
    let mut addresses = Vec::with_capacity(length);
    for i in 0..length {
        let start = 128 + i * 64;
        if result.len() < start + 64 {
            break;
        }
        let w_bytes: [u8; 32] = hex::decode(&result[start..start + 64])
            .unwrap_or_default()
            .try_into()
            .unwrap_or([0u8; 32]);
        addresses.push(word_to_address(&w_bytes));
    }
    Ok(addresses)
}

/// Fetch vault info for multiple vaults in a single resolver call.
/// Uses getVaultsEntireData(address[]) which returns packed fixed-size structs.
pub async fn vault_infos_batch(chain_id: u64, addresses: &[String]) -> anyhow::Result<Vec<VaultInfo>> {
    if addresses.is_empty() {
        return Ok(vec![]);
    }
    let sel = selector("getVaultsEntireData(address[])");
    let addr_refs: Vec<&str> = addresses.iter().map(|s| s.as_str()).collect();
    let payload = encode_address_array(&addr_refs);
    let data = calldata_raw(sel, &payload);

    let result = eth_call(chain_id, VAULT_RESOLVER, &data).await?;

    // ABI: offset(32) + length + N × VAULT_STRUCT_WORDS words
    if result.len() < 128 {
        return Ok(vec![]);
    }
    let length = usize::from_str_radix(&result[64..128], 16).unwrap_or(0);
    let mut infos = Vec::with_capacity(length);

    for i in 0..length {
        let base = 128 + i * VAULT_STRUCT_WORDS * 64;
        if result.len() < base + VAULT_STRUCT_WORDS * 64 {
            break;
        }
        let get = |slot: usize| -> Option<[u8; 32]> {
            let start = base + slot * 64;
            let mut w = [0u8; 32];
            hex::decode_to_slice(&result[start..start + 64], &mut w).ok()?;
            Some(w)
        };

        let is_smart_col = get(WORD_IS_SMART_COL).map(|w| word_to_bool(&w)).unwrap_or(false);
        let is_smart_debt = get(WORD_IS_SMART_DEBT).map(|w| word_to_bool(&w)).unwrap_or(false);
        let col_token = get(WORD_COL_TOKEN).map(|w| word_to_address(&w)).unwrap_or_default();
        let debt_token = get(WORD_DEBT_TOKEN).map(|w| word_to_address(&w)).unwrap_or_default();

        infos.push(VaultInfo {
            address: addresses[i].clone(),
            is_smart_col,
            is_smart_debt,
            col_token,
            debt_token,
        });
    }
    Ok(infos)
}

/// Fetch single vault info.
pub async fn vault_info_single(chain_id: u64, vault: &str) -> anyhow::Result<VaultInfo> {
    let sel = selector("getVaultEntireData(address)");
    let data = calldata(sel, &[encode_address(vault)]);
    let result = eth_call(chain_id, VAULT_RESOLVER, &data).await?;

    if result.len() < VAULT_STRUCT_WORDS * 64 {
        anyhow::bail!("Unexpected vault data length for {}", vault);
    }
    let get = |slot: usize| -> Option<[u8; 32]> { word(&result, slot) };

    Ok(VaultInfo {
        address: vault.to_lowercase(),
        is_smart_col: get(WORD_IS_SMART_COL).map(|w| word_to_bool(&w)).unwrap_or(false),
        is_smart_debt: get(WORD_IS_SMART_DEBT).map(|w| word_to_bool(&w)).unwrap_or(false),
        col_token: get(WORD_COL_TOKEN).map(|w| word_to_address(&w)).unwrap_or_default(),
        debt_token: get(WORD_DEBT_TOKEN).map(|w| word_to_address(&w)).unwrap_or_default(),
    })
}

/// Vault type label.
pub fn vault_type(info: &VaultInfo) -> &'static str {
    match (info.is_smart_col, info.is_smart_debt) {
        (false, false) => "T1",
        (true, false)  => "T2",
        (false, true)  => "T2",
        (true, true)   => "T3",
    }
}

/// Get the vault address for a given NFT ID.
pub async fn vault_for_nft(chain_id: u64, nft_id: u64) -> anyhow::Result<String> {
    let sel = selector("getVaultAddressFromNftId(uint256)");
    let data = calldata(sel, &[{
        let mut w = [0u8; 32];
        w[24..].copy_from_slice(&nft_id.to_be_bytes());
        w
    }]);
    let result = eth_call(chain_id, VAULT_RESOLVER, &data).await?;
    let w = word(&result, 0).ok_or_else(|| anyhow::anyhow!("No result for vault_for_nft"))?;
    Ok(word_to_address(&w))
}
