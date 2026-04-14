"""
一键发币 v1.0 — Four.Meme adapter (BSC, direct contract interaction).

Flow:
  1. Upload image + metadata to IPFS (Pinata)
  2. Call Four.Meme factory contract via onchainos wallet contract-call
  3. Include msg.value for initial buy (bundled)
  4. Wait for confirmation

Four.Meme is the largest BSC launchpad. It doesn't expose a public REST API
for token creation, so we interact with the factory contract directly.
"""
from __future__ import annotations

import asyncio
import json

import config as C
from .base import LaunchpadAdapter, LaunchParams, LaunchResult, onchainos_bin

_BSC_EXPLORER = "https://bscscan.com/tx"
_FOURMEME_TRADE = "https://four.meme/token"


class FourMemeAdapter(LaunchpadAdapter):

    @property
    def name(self) -> str:
        return "fourmeme"

    @property
    def display_name(self) -> str:
        return "Four.Meme"

    @property
    def chain(self) -> str:
        return "bsc"

    def _fee_estimate(self, params: LaunchParams) -> float:
        return 0.015  # BNB gas

    async def launch(self, params: LaunchParams) -> LaunchResult:
        """Launch a token on Four.Meme (BSC)."""

        if C.DRY_RUN:
            return LaunchResult(
                success=True,
                token_address="DRY_RUN_FOURMEME_NO_TOKEN",
                tx_hash="DRY_RUN_FOURMEME_NO_TX",
                error="DRY_RUN mode — no on-chain TX sent",
            )

        factory = C.FOURMEME_FACTORY
        if not factory:
            return LaunchResult(
                success=False,
                error="FOURMEME_FACTORY address not configured in config.py. "
                      "Set the Four.Meme factory contract address.",
            )

        category = params.extras.get("category", C.FOURMEME_CATEGORY)
        gas_price = params.extras.get("gas_price", C.FOURMEME_GAS_PRICE)

        # Build contract call data
        # Four.Meme createToken function — ABI encoded
        # The exact function signature depends on the factory version
        # Typical: createToken(string name, string symbol, string metadataURI, uint256 category)
        # with msg.value = buyAmount (in BNB wei)

        buy_wei = int(params.buy_amount * 10**18) if params.buy_amount > 0 else 0

        # Use onchainos wallet contract-call for BSC
        print("  [Four.Meme] Calling factory contract...")

        cmd = [
            onchainos_bin(), "wallet", "contract-call",
            "--chain", "bsc",
            "--to", factory,
            "--function", "createToken(string,string,string,string)",
            "--args", json.dumps([
                params.name,
                params.symbol,
                params.metadata_uri,
                category,
            ]),
        ]

        if buy_wei > 0:
            cmd.extend(["--value", str(buy_wei)])

        if gas_price:
            cmd.extend(["--gas-price", gas_price])

        try:
            proc = await asyncio.create_subprocess_exec(
                *cmd,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
            )
            stdout, stderr = await proc.communicate()

            if proc.returncode != 0:
                err = stderr.decode().strip() if stderr else "unknown error"
                return LaunchResult(
                    success=False,
                    error=f"Four.Meme contract-call failed: {err}",
                )

            output = json.loads(stdout.decode())
            tx_hash = output.get("data", {}).get("txHash", "") or output.get("txHash", "")

        except Exception as e:
            return LaunchResult(success=False, error=f"Four.Meme launch error: {e}")

        if not tx_hash:
            return LaunchResult(success=False, error="No tx hash returned from contract-call")

        # ── Wait for confirmation (~3-5s on BSC) ──────────────────────
        print(f"  [Four.Meme] TX submitted: {tx_hash}")
        confirmed, token_address = await self._wait_and_parse(tx_hash)

        return LaunchResult(
            success=confirmed,
            token_address=token_address,
            tx_hash=tx_hash,
            explorer_url=f"{_BSC_EXPLORER}/{tx_hash}",
            trade_page_url=f"{_FOURMEME_TRADE}/{token_address}" if token_address else "",
            error="" if confirmed else "Transaction not confirmed within timeout",
        )

    async def _wait_and_parse(self, tx_hash: str, max_retries: int = 6) -> tuple:
        """Wait for BSC TX confirmation and parse token address from logs.

        Returns (confirmed: bool, token_address: str).
        """
        for i in range(max_retries):
            await asyncio.sleep(3)  # BSC is ~3s blocks
            try:
                proc = await asyncio.create_subprocess_exec(
                    onchainos_bin(), "gateway", "tx-status",
                    "--chain", "bsc",
                    "--tx-hash", tx_hash,
                    stdout=asyncio.subprocess.PIPE,
                    stderr=asyncio.subprocess.PIPE,
                )
                stdout, _ = await proc.communicate()
                output = json.loads(stdout.decode())
                data = output.get("data", {})
                status = data.get("status", "")

                if status in ("confirmed", "finalized", "success", "1"):
                    # Try to extract token address from logs
                    token_addr = ""
                    logs = data.get("logs", [])
                    for log in logs:
                        # TokenCreated event contains the new token address
                        topics = log.get("topics", [])
                        if len(topics) >= 2:
                            # Token address is typically in the event data
                            addr = log.get("address", "")
                            if addr and addr != C.FOURMEME_FACTORY:
                                token_addr = addr
                                break

                    # Fallback: check contractAddress in receipt
                    if not token_addr:
                        token_addr = data.get("contractAddress", "")

                    print(f"  [Four.Meme] Confirmed! Token: {token_addr or 'parsing...'}")
                    return True, token_addr

            except Exception:
                pass

        return False, ""
