# puffer
Liquid restaking on Puffer Finance (Ethereum) — deposit ETH to mint pufETH, choose between instant 1-step withdraw (1% exit fee) or 2-step queued withdraw (~14d, fee-free), and check balances/rate.

## Highlights
- Deposit ETH into PufferVault to mint pufETH (ERC-4626 nLRT)
- Two exit paths with full cost/delivery info in each command's structured JSON output:
  - `instant-withdraw` — 1 tx, immediate WETH, pays the live exit fee (default 1%)
  - `request-withdraw` → `claim-withdraw` — 2-step queued, no fee, ~14 days
- `withdraw-options` command compares both paths before you commit
- `withdraw-status` command tracks a queued withdrawal by index (PENDING / CLAIMABLE / ALREADY_CLAIMED / OUT_OF_RANGE)
- Read-only `positions` + `rate` views with live APY from DeFiLlama
- All failures emit stable `error_code` to stdout so external agents can branch deterministically
- Two-step confirmation gate (`--confirm`) on every write operation
- Built-in pre-flight balance checks, proper `wait_for_tx` on `approve` (no sleep-based races)
