"""
一键发币 v1.0 — Flap.sh adapter (BSC, direct contract interaction).

Flow:
  1. Upload image + metadata to IPFS (Pinata)
  2. Call Flap Portal contract newTokenV6() via onchainos wallet contract-call
  3. Supports tax tokens (buy/sell tax), vanity addresses, PCS V2/V3 migration
  4. Wait for confirmation

Portal: 0xe2cE6ab80874Fa9Fa2aAE65D277Dd6B8e65C9De0 (BNB Mainnet)
Docs:   https://docs.flap.sh/flap/developers/launch-a-token
"""
from __future__ import annotations

import asyncio
import json

import config as C
from .base import LaunchpadAdapter, LaunchParams, LaunchResult, onchainos_bin

_BSC_EXPLORER = "https://bscscan.com/tx"
_FLAP_TRADE = "https://flap.sh/token"
_ZERO_ADDR = "0x0000000000000000000000000000000000000000"
_ZERO_BYTES32 = "0x" + "00" * 32


class FlapAdapter(LaunchpadAdapter):

    @property
    def name(self) -> str:
        return "flap"

    @property
    def display_name(self) -> str:
        return "Flap.sh"

    @property
    def chain(self) -> str:
        return "bsc"

    def _fee_estimate(self, params: LaunchParams) -> float:
        return 0.015  # BNB gas

    async def launch(self, params: LaunchParams) -> LaunchResult:
        """Launch a token on Flap.sh (BSC) via newTokenV6."""

        if C.DRY_RUN:
            return LaunchResult(
                success=True,
                token_address="DRY_RUN_FLAP_NO_TOKEN",
                tx_hash="DRY_RUN_FLAP_NO_TX",
                error="DRY_RUN mode — no on-chain TX sent",
            )

        portal = C.FLAP_PORTAL
        extras = params.extras

        # ── Build newTokenV6 parameters ───────────────────────────────
        buy_tax = extras.get("buy_tax", C.FLAP_BUY_TAX)
        sell_tax = extras.get("sell_tax", C.FLAP_SELL_TAX)
        tax_duration = extras.get("tax_duration", C.FLAP_TAX_DURATION)
        anti_farmer = extras.get("anti_farmer", C.FLAP_ANTI_FARMER)
        migrator_type = extras.get("migrator_type", C.FLAP_MIGRATOR_TYPE)
        dex_id = extras.get("dex_id", C.FLAP_DEX_ID)
        lp_fee_profile = extras.get("lp_fee_profile", C.FLAP_LP_FEE_PROFILE)
        token_version = extras.get("token_version", C.FLAP_TOKEN_VERSION)
        salt = extras.get("salt", _ZERO_BYTES32)
        beneficiary = extras.get("beneficiary", params.wallet_address)

        # Tax allocation split
        mkt_bps = extras.get("mkt_bps", C.FLAP_MKT_BPS)
        deflation_bps = extras.get("deflation_bps", C.FLAP_DEFLATION_BPS)
        dividend_bps = extras.get("dividend_bps", C.FLAP_DIVIDEND_BPS)
        lp_bps = extras.get("lp_bps", C.FLAP_LP_BPS)

        buy_wei = int(params.buy_amount * 10**18) if params.buy_amount > 0 else 0

        # newTokenV6 struct parameter (ABI-encoded as tuple)
        # We pass all fields as a JSON array for onchainos contract-call
        v6_params = {
            "name": params.name,
            "symbol": params.symbol,
            "meta": params.metadata_cid,  # IPFS CID (not full URI)
            "dexThresh": 0,               # Default DEX listing threshold
            "salt": salt,
            "migratorType": migrator_type,
            "quoteToken": _ZERO_ADDR,     # address(0) = native BNB
            "quoteAmt": buy_wei,
            "beneficiary": beneficiary,
            "permitData": "0x",
            "extensionID": _ZERO_BYTES32,
            "extensionData": "0x",
            "dexId": dex_id,
            "lpFeeProfile": lp_fee_profile,
            "buyTaxRate": buy_tax,
            "sellTaxRate": sell_tax,
            "taxDuration": tax_duration,
            "antiFarmerDuration": anti_farmer,
            "mktBps": mkt_bps,
            "deflationBps": deflation_bps,
            "dividendBps": dividend_bps,
            "lpBps": lp_bps,
            "minimumShareBalance": 0,
            "dividendToken": _ZERO_ADDR,
            "commissionReceiver": _ZERO_ADDR,
            "tokenVersion": token_version,
        }

        print(f"  [Flap] Calling newTokenV6 on portal {portal[:10]}...")
        if buy_tax > 0 or sell_tax > 0:
            print(f"  [Flap] Tax config: buy={buy_tax}bps sell={sell_tax}bps duration={tax_duration}s")
        if salt != _ZERO_BYTES32:
            print(f"  [Flap] Vanity salt: {salt[:10]}...")

        # Build onchainos contract-call command
        # newTokenV6 takes a struct param — we encode as tuple
        func_sig = (
            "newTokenV6("
            "(string,string,string,uint8,bytes32,uint8,address,uint256,"
            "address,bytes,bytes32,bytes,uint8,uint8,"
            "uint16,uint16,uint256,uint256,"
            "uint16,uint16,uint16,uint16,"
            "uint256,address,address,uint8))"
        )

        args_tuple = [
            params.name,             # name
            params.symbol,           # symbol
            params.metadata_cid,     # meta (IPFS CID)
            0,                       # dexThresh
            salt,                    # salt
            migrator_type,           # migratorType
            _ZERO_ADDR,             # quoteToken
            buy_wei,                # quoteAmt
            beneficiary,            # beneficiary
            "0x",                   # permitData
            _ZERO_BYTES32,          # extensionID
            "0x",                   # extensionData
            dex_id,                 # dexId
            lp_fee_profile,         # lpFeeProfile
            buy_tax,                # buyTaxRate
            sell_tax,               # sellTaxRate
            tax_duration,           # taxDuration
            anti_farmer,            # antiFarmerDuration
            mkt_bps,                # mktBps
            deflation_bps,          # deflationBps
            dividend_bps,           # dividendBps
            lp_bps,                 # lpBps
            0,                      # minimumShareBalance
            _ZERO_ADDR,            # dividendToken
            _ZERO_ADDR,            # commissionReceiver
            token_version,          # tokenVersion
        ]

        cmd = [
            onchainos_bin(), "wallet", "contract-call",
            "--chain", "bsc",
            "--to", portal,
            "--function", func_sig,
            "--args", json.dumps([args_tuple]),
        ]

        if buy_wei > 0:
            cmd.extend(["--value", str(buy_wei)])

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
                    error=f"Flap contract-call failed: {err}",
                )

            output = json.loads(stdout.decode())
            tx_hash = output.get("data", {}).get("txHash", "") or output.get("txHash", "")

        except Exception as e:
            return LaunchResult(success=False, error=f"Flap launch error: {e}")

        if not tx_hash:
            return LaunchResult(success=False, error="No tx hash returned from contract-call")

        # ── Wait for BSC confirmation ─────────────────────────────────
        print(f"  [Flap] TX submitted: {tx_hash}")
        confirmed, token_address = await self._wait_and_parse(tx_hash)

        return LaunchResult(
            success=confirmed,
            token_address=token_address,
            tx_hash=tx_hash,
            explorer_url=f"{_BSC_EXPLORER}/{tx_hash}",
            trade_page_url=f"{_FLAP_TRADE}/{token_address}" if token_address else "",
            error="" if confirmed else "Transaction not confirmed within timeout",
        )

    async def _wait_and_parse(self, tx_hash: str, max_retries: int = 6) -> tuple:
        """Wait for BSC TX confirmation and extract token address.

        Flap emits TokenCreated(ts, creator, nonce, token, name, symbol, meta)
        The `token` parameter is the new token address.
        """
        for i in range(max_retries):
            await asyncio.sleep(3)
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
                    # Parse TokenCreated event from logs
                    token_addr = ""
                    logs = data.get("logs", [])
                    for log in logs:
                        topics = log.get("topics", [])
                        # TokenCreated event — token address in the data field
                        if len(topics) >= 1 and log.get("address", "").lower() == C.FLAP_PORTAL.lower():
                            log_data = log.get("data", "")
                            # Token address is typically at a known offset in the event data
                            if len(log_data) >= 130:  # 0x + 64 chars offset
                                # Parse address from data (offset varies by event structure)
                                addr_hex = log_data[90:130]  # Token address field
                                token_addr = "0x" + addr_hex[-40:]

                    if not token_addr:
                        token_addr = data.get("contractAddress", "")

                    print(f"  [Flap] Confirmed! Token: {token_addr or 'parsing...'}")
                    return True, token_addr

            except Exception:
                pass

        return False, ""
