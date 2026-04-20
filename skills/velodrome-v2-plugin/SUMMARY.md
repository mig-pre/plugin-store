**Overview**

Swap tokens and provide liquidity on Velodrome V2's AMM on Optimism — supporting volatile and stable pool types — earning trading fees and VELO emissions from pool gauges.

**Prerequisites**
- onchainos agentic wallet connected
- Optimism wallet (chain 10) with tokens to swap or provide as liquidity
- For `add-liquidity`: both tokens of the pair in your wallet
- For `claim-rewards`: an LP position in a pool that has a gauge

**How it Works**

Swapping:
1. **Get a quote**: Check the expected output before committing — auto-checks both volatile and stable pools (no gas). `velodrome-v2-plugin quote --token-in WETH --token-out USDC --amount-in 0.1`
2. **Execute the swap**: Send tokens and receive output in one transaction. `velodrome-v2-plugin swap --token-in WETH --token-out USDC --amount-in 0.1 --slippage 0.5 --confirm`

Providing liquidity:
3. **Check the pool**: Verify the pair exists and see pool type (volatile or stable). `velodrome-v2-plugin pools --token-a WETH --token-b USDC`
4. **Add liquidity**: Deposit both tokens to receive LP tokens — use `--stable` flag for stable pools (USDC/DAI). `velodrome-v2-plugin add-liquidity --token-a WETH --token-b USDC --amount-a-desired 0.001 --confirm`
5. **View LP balance**: Check your current LP position in the pool. `velodrome-v2-plugin positions --token-a WETH --token-b USDC`
6. **Remove liquidity**: Withdraw your LP tokens and receive both underlying tokens back. `velodrome-v2-plugin remove-liquidity --token-a WETH --token-b USDC --confirm`
7. **Claim VELO rewards**: Collect VELO emissions from pool gauges — requires an LP position in an incentivized pool. `velodrome-v2-plugin claim-rewards --token-a WETH --token-b USDC --confirm`
