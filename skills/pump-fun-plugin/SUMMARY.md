**Overview**

Buy and sell tokens on pump.fun bonding curves from the CLI — check token info, bonding curve progress, and price quotes before any on-chain action.

**Prerequisites**
- onchainos agentic wallet connected
- Some SOL in your wallet for the buy amount plus network fees

**How it Works**
1. **Research**: Look up a token before trading — active bonding curve tokens end in `pump`.
   - 1.1 **Token info**: See bonding curve reserves, current price, and graduation progress. `pump-fun-plugin get-token-info --mint <TOKEN_MINT>`
   - 1.2 **Get a price quote**: Check the expected cost before buying or expected proceeds before selling. `pump-fun-plugin get-price --mint <TOKEN_MINT> --direction buy`
2. **Buy**:
   - 2.1 **Preview**: See the transaction details without sending — no gas. `pump-fun-plugin buy --mint <TOKEN_MINT> --sol-amount <amount>`
   - 2.2 **Execute**: Purchase tokens from the bonding curve. `pump-fun-plugin buy --mint <TOKEN_MINT> --sol-amount <amount> --confirm`
3. **Sell**:
   - 3.1 **Preview**: See expected SOL proceeds before selling. `pump-fun-plugin sell --mint <TOKEN_MINT> --token-amount <AMOUNT>`
   - 3.2 **Execute**: Sell tokens back to the bonding curve — omit `--token-amount` to sell your full balance. `pump-fun-plugin sell --mint <TOKEN_MINT> --token-amount <AMOUNT> --confirm`
