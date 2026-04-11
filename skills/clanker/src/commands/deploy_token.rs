// src/commands/deploy_token.rs — deploy via direct factory deployToken() call (no API key required)
//
// Calls Clanker V4 factory `deployToken(DeploymentConfig)` directly from the user's wallet.
// Factory: 0xE85A59c628F7d27878ACeB4bf3b35733630083a9 (Base)
// ABI source: github.com/clanker-devco/clanker-sdk — src/abi/v4/Clanker.ts

#![allow(non_snake_case)]

use alloy_primitives::{Address, Bytes, FixedBytes, U256};
use alloy_sol_types::{sol, SolCall, SolValue};
use anyhow::{bail, Result};
use uuid::Uuid;

use crate::config;
use crate::onchainos;

// ── Addresses (Base, chain 8453) ───────────────────────────────────────────

const WETH_BASE: &str            = "0x4200000000000000000000000000000000000006";
const HOOK_STATIC_V2_BASE: &str  = "0xb429d62f8f3bFFb98CdB9569533eA23bF0Ba28CC";
const LOCKER_BASE: &str          = "0x63D2DfEA64b3433F4071A98665bcD7Ca14d93496";
const MEV_MODULE_V2_BASE: &str   = "0xebB25BB797D82CB78E1bc70406b13233c0854413";

// ── Pool parameters ────────────────────────────────────────────────────────

// tick when token address < WETH address (token0 = clanker token)
// -230400 ≈ 10 ETH initial market cap
const TICK_IF_TOKEN0_IS_CLANKER: i32 = -230400;
const TICK_SPACING: i32              = 200;

// LP position range: one-sided (token-only), starting just below initial tick
const TICK_LOWER: i32 = -230400;
const TICK_UPPER: i32 = -120000;

// Fee in uniBps (bps × 100). 100 bps = 1% fee → 10000 uniBps.
const FEE_UNI_BPS: u32 = 10_000;

// MEV module defaults (gradual fee decay over ~15 seconds after each block)
const MEV_STARTING_FEE: u32 = 666_777;
const MEV_ENDING_FEE: u32   = 41_673;
const MEV_DECAY_SECS: u64   = 15;

// ── ABI types ──────────────────────────────────────────────────────────────

sol! {
    // ── Inner encoding structs (bytes fields) ──────────────────────────────

    /// Encodes into PoolConfig.poolData for feeStaticHookV2
    struct PoolInitializationData {
        address extension;
        bytes   extensionData;
        bytes   feeData;
    }

    /// Fee parameters encoded into PoolInitializationData.feeData
    struct FeeConfig {
        uint24 clankerFee;
        uint24 pairedFee;
    }

    /// Encodes into LockerConfig.lockerData
    struct LockerInstantiationData {
        uint8[] feePreference;
    }

    /// Encodes into MevModuleConfig.mevModuleData
    struct MevSniperAuctionInitData {
        uint24  startingFee;
        uint24  endingFee;
        uint256 secondsToDecay;
    }

    // ── Factory ABI structs ────────────────────────────────────────────────

    struct TokenConfig {
        address tokenAdmin;
        string  name;
        string  symbol;
        bytes32 salt;
        string  image;
        string  metadata;
        string  context;
        uint256 originatingChainId;
    }

    struct PoolConfig {
        address hook;
        address pairedToken;
        int24   tickIfToken0IsClanker;
        int24   tickSpacing;
        bytes   poolData;
    }

    struct LockerConfig {
        address   locker;
        address[] rewardAdmins;
        address[] rewardRecipients;
        uint16[]  rewardBps;
        int24[]   tickLower;
        int24[]   tickUpper;
        uint16[]  positionBps;
        bytes     lockerData;
    }

    struct MevModuleConfig {
        address mevModule;
        bytes   mevModuleData;
    }

    struct ExtensionConfig {
        address extension;
        uint256 msgValue;
        uint16  extensionBps;
        bytes   extensionData;
    }

    struct DeploymentConfig {
        TokenConfig       tokenConfig;
        PoolConfig        poolConfig;
        LockerConfig      lockerConfig;
        MevModuleConfig   mevModuleConfig;
        ExtensionConfig[] extensionConfigs;
    }

    function deployToken(DeploymentConfig deploymentConfig)
        external payable returns (address tokenAddress);
}

// ── Type aliases ───────────────────────────────────────────────────────────
type I24 = alloy_primitives::aliases::I24;
type U24 = alloy_primitives::aliases::U24;

/// Helper: convert an i32 into an alloy I24 (int24), bailing on overflow.
fn i24(v: i32) -> Result<I24> {
    I24::try_from(v as i64).map_err(|_| anyhow::anyhow!("int24 overflow: {}", v))
}

/// Helper: convert a u32 into an alloy U24 (uint24), bailing on overflow.
fn u24(v: u32) -> Result<U24> {
    U24::try_from(v as u64).map_err(|_| anyhow::anyhow!("uint24 overflow: {}", v))
}

#[allow(clippy::too_many_arguments)]
pub async fn run(
    chain_id: u64,
    name: &str,
    symbol: &str,
    from: Option<&str>,
    image_url: Option<&str>,
    dry_run: bool,
) -> Result<()> {
    // Only Base supported for direct on-chain deployment
    if chain_id != 8453 {
        bail!(
            "Direct on-chain deployment is only supported on Base (chain 8453). \
             Arbitrum support is planned for a future release."
        );
    }

    // ── 1. Resolve wallet ─────────────────────────────────────────────────
    let wallet_str = from
        .map(|s| s.to_string())
        .unwrap_or_else(|| onchainos::resolve_wallet(chain_id).unwrap_or_default());
    if wallet_str.is_empty() {
        bail!("Cannot determine wallet address — pass --from or ensure onchainos is logged in");
    }

    let factory = config::factory_address(chain_id)
        .ok_or_else(|| anyhow::anyhow!("No factory address configured for chain {}", chain_id))?;

    let wallet_addr: Address = wallet_str
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid wallet address: {}", wallet_str))?;

    let hook_addr: Address    = HOOK_STATIC_V2_BASE.parse().unwrap();
    let weth_addr: Address    = WETH_BASE.parse().unwrap();
    let locker_addr: Address  = LOCKER_BASE.parse().unwrap();
    let mev_addr: Address     = MEV_MODULE_V2_BASE.parse().unwrap();

    // ── 2. Derive unique salt from a UUID (prevents address collisions) ───
    let uuid = Uuid::new_v4();
    let mut salt_bytes = [0u8; 32];
    salt_bytes[..16].copy_from_slice(uuid.as_bytes());
    let salt = FixedBytes::<32>::from(salt_bytes);

    // ── 3. Encode inner bytes fields ──────────────────────────────────────

    // feeData = abi.encode(uint24(10000), uint24(10000))
    let fee_data = FeeConfig {
        clankerFee: u24(FEE_UNI_BPS)?,
        pairedFee:  u24(FEE_UNI_BPS)?,
    }
    .abi_encode();

    // poolData = abi.encode(PoolInitializationData{zeroAddr, 0x, feeData})
    let pool_data = PoolInitializationData {
        extension:     Address::ZERO,
        extensionData: Bytes::new(),
        feeData:       Bytes::from(fee_data),
    }
    .abi_encode();

    // lockerData = abi.encode(LockerInstantiationData{feePreference:[0]})
    // feePreference 0 = Both (take fees in both WETH and token)
    let locker_data = LockerInstantiationData {
        feePreference: vec![0u8],
    }
    .abi_encode();

    // mevModuleData = abi.encode(MevSniperAuctionInitData{...})
    let mev_data = MevSniperAuctionInitData {
        startingFee:    u24(MEV_STARTING_FEE)?,
        endingFee:      u24(MEV_ENDING_FEE)?,
        secondsToDecay: U256::from(MEV_DECAY_SECS),
    }
    .abi_encode();

    // ── 4. Assemble DeploymentConfig ──────────────────────────────────────

    let deployment_config = DeploymentConfig {
        tokenConfig: TokenConfig {
            tokenAdmin:          wallet_addr,
            name:                name.to_string(),
            symbol:              symbol.to_string(),
            salt,
            image:               image_url.unwrap_or("").to_string(),
            metadata:            String::new(),
            context:             String::new(),
            originatingChainId:  U256::from(chain_id),
        },
        poolConfig: PoolConfig {
            hook:                  hook_addr,
            pairedToken:           weth_addr,
            tickIfToken0IsClanker: i24(TICK_IF_TOKEN0_IS_CLANKER)?,
            tickSpacing:           i24(TICK_SPACING)?,
            poolData:              Bytes::from(pool_data),
        },
        lockerConfig: LockerConfig {
            locker:           locker_addr,
            rewardAdmins:     vec![wallet_addr],
            rewardRecipients: vec![wallet_addr],
            rewardBps:        vec![10_000u16],
            tickLower:        vec![i24(TICK_LOWER)?],
            tickUpper:        vec![i24(TICK_UPPER)?],
            positionBps:      vec![10_000u16],
            lockerData:       Bytes::from(locker_data),
        },
        mevModuleConfig: MevModuleConfig {
            mevModule:    mev_addr,
            mevModuleData: Bytes::from(mev_data),
        },
        extensionConfigs: vec![],
    };

    // ── 5. Encode calldata ────────────────────────────────────────────────

    let calldata = format!(
        "0x{}",
        hex::encode(deployTokenCall { deploymentConfig: deployment_config }.abi_encode())
    );

    // ── 6. Dry-run preview ────────────────────────────────────────────────
    if dry_run {
        let preview = serde_json::json!({
            "ok": true,
            "dry_run": true,
            "data": {
                "action": "deploy_token",
                "chain_id": chain_id,
                "name": name,
                "symbol": symbol,
                "token_admin": wallet_str,
                "reward_recipient": wallet_str,
                "paired_token": "WETH",
                "hook": "feeStaticHookV2",
                "mev_protection": "mevModuleV2 (gradual fee decay)",
                "initial_price_tick": TICK_IF_TOKEN0_IS_CLANKER,
                "lp_range": { "tick_lower": TICK_LOWER, "tick_upper": TICK_UPPER },
                "factory": factory,
                "calldata_selector": &calldata[..10],
                "note": "Run without --dry-run after user confirmation to execute on-chain"
            }
        });
        println!("{}", serde_json::to_string_pretty(&preview)?);
        return Ok(());
    }

    // ── 7. Execute on-chain ───────────────────────────────────────────────
    // The agent MUST ask user to confirm before calling this command without --dry-run.
    let result = onchainos::wallet_contract_call(
        chain_id,
        factory,
        &calldata,
        Some(&wallet_str),
        None,   // msg.value = 0 (no devBuy extension)
        true,   // --force: prevents "pending" txHash
        false,
    )
    .await?;

    let tx_hash = onchainos::extract_tx_hash_or_err(&result)?;

    let output = serde_json::json!({
        "ok": true,
        "data": {
            "name": name,
            "symbol": symbol,
            "chain_id": chain_id,
            "token_admin": wallet_str,
            "reward_recipient": wallet_str,
            "tx_hash": tx_hash,
            "explorer_url": format!("https://basescan.org/tx/{}", tx_hash),
            "note": "Token deployment submitted. Check the transaction on Basescan to find the deployed contract address (look for the contract creation event or Transfer from address(0))."
        }
    });
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
