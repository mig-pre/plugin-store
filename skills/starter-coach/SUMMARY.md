# starter-coach
Conversational 6-step skill that guides users to build their own automated DEX spot-trading bot on OKX DEX — from zero to live in one session, with paper trading gate and legal disclaimers built in.

## Highlights
- 6-step guided flow: Onboard → Profile → Build Strategy → Backtest → Paper Trade → Go Live
- 7 goal archetypes: Stack Sats, Buy the Dip, Follow Smart Money, Copy-trade Wallet, Snipe Tokens, Ride the Trend, Something Else
- Bilingual support: English and Chinese (auto-detected)
- Backtesting with disclaimer: "Past performance does not guarantee future results"
- Paper trading gate — users must pass paper trade before going live
- Live mode requires explicit "CONFIRM" acknowledgment, logged to audit file
- Kelly-based position sizing + configurable risk parameters
- All on-chain execution via OnchainOS Agentic Wallet (TEE signing, no API key needed)
- Validated JSON strategy specs — no freeform trading code generated
