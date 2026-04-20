**Overview**

Swap tokens and manage liquidity on PancakeSwap V2's constant-product AMM (0.25% fee) across BSC, Base, and Arbitrum — LP tokens are standard ERC-20 and composable with other DeFi protocols.

**Prerequisites**
- onchainos agentic wallet connected
- Some tokens on a supported chain — BSC (default), Base, or Arbitrum

**How it Works**
1. **Swap**:
   - 1.1 **Get a quote**: Check the expected output before committing — no gas. `pancakeswap-v2-plugin quote --token-in USDT --token-out CAKE --amount-in <amount>`
   - 1.2 **Execute the swap**: Send input token and receive output — ERC-20 approval fires automatically if needed. `pancakeswap-v2-plugin swap --token-in USDT --token-out CAKE --amount-in <amount> --confirm`
2. **Provide liquidity**:
   - 2.1 **Look up a pair**: Find the LP contract address for a token pair. `pancakeswap-v2-plugin get-pair --token-a CAKE --token-b BNB`
   - 2.2 **Check reserves**: See the current token balances and implied price in a pair. `pancakeswap-v2-plugin get-reserves --pair <address>`
   - 2.3 **Add liquidity**: Deposit both tokens of the pair to receive LP tokens. `pancakeswap-v2-plugin add-liquidity --token-a CAKE --token-b BNB --amount-a <amount-a> --amount-b <amount-b> --confirm`
   - 2.4 **Check LP balance**: View your LP token holdings for a pair. `pancakeswap-v2-plugin lp-balance --pair <address>`
   - 2.5 **Remove liquidity**: Burn LP tokens to withdraw your proportional share of the pool. `pancakeswap-v2-plugin remove-liquidity --pair <address> --liquidity <amount> --confirm`
