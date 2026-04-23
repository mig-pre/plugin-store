use crate::config::{pad_address, pad_u256};

// ============================================================
// PufferVaultV5 — mint pufETH
// ============================================================

/// PufferVaultV5.depositETH(address receiver) payable
/// Selector: 0x2d2da806
pub fn build_deposit_eth_calldata(receiver: &str) -> String {
    format!("0x2d2da806{}", pad_address(receiver))
}

/// PufferVaultV5.deposit(uint256 assets, address receiver) — WETH path (ERC-4626).
/// Selector: 0x6e553f65
/// Reserved for v0.2.x (WETH stake command). Not wired yet.
#[allow(dead_code)]
pub fn build_deposit_weth_calldata(assets: u128, receiver: &str) -> String {
    format!(
        "0x6e553f65{}{}",
        pad_u256(assets),
        pad_address(receiver),
    )
}

// ============================================================
// PufferVaultV5 — 1-step instant withdraw (applies exit fee)
// ============================================================

/// PufferVaultV5.redeem(uint256 shares, address receiver, address owner)
/// Selector: 0xba087652
/// Burns `shares` pufETH, transfers WETH (assets minus exit fee) to receiver.
pub fn build_redeem_calldata(shares: u128, receiver: &str, owner: &str) -> String {
    format!(
        "0xba087652{}{}{}",
        pad_u256(shares),
        pad_address(receiver),
        pad_address(owner),
    )
}

/// PufferVaultV5.withdraw(uint256 assets, address receiver, address owner)
/// Selector: 0xb460af94
/// Specify WETH amount out; pulls up to `previewWithdraw(assets)` pufETH from owner.
#[allow(dead_code)]
pub fn build_withdraw_assets_calldata(assets: u128, receiver: &str, owner: &str) -> String {
    format!(
        "0xb460af94{}{}{}",
        pad_u256(assets),
        pad_address(receiver),
        pad_address(owner),
    )
}

// ============================================================
// PufferWithdrawalManager — 2-step queued withdraw (no fee)
// ============================================================

/// PufferWithdrawalManager.requestWithdrawal(uint128 pufETHAmount, address recipient)
/// Selector: 0xef027fbf
/// Note: pufETHAmount is uint128 but ABI-encoded as 32 bytes (left-padded).
pub fn build_request_withdrawal_calldata(pufeth_amount: u128, recipient: &str) -> String {
    format!(
        "0xef027fbf{}{}",
        pad_u256(pufeth_amount),
        pad_address(recipient),
    )
}

/// PufferWithdrawalManager.completeQueuedWithdrawal(uint256 withdrawalIdx)
/// Selector: 0x6a4800a4
pub fn build_complete_queued_withdrawal_calldata(idx: u128) -> String {
    format!("0x6a4800a4{}", pad_u256(idx))
}

// ============================================================
// ERC-20 approve (shared helper)
// ============================================================

/// ERC-20 approve(address spender, uint256 amount)
/// Selector: 0x095ea7b3
pub fn build_approve_calldata(spender: &str, amount: u128) -> String {
    format!(
        "0x095ea7b3{}{}",
        pad_address(spender),
        pad_u256(amount),
    )
}
