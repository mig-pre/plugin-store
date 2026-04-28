use clap::Args;
use serde_json::{json, Value};

use crate::api::{self, QuoteParams};
use crate::config::{is_native_token, parse_chain, supported_chains_help, NATIVE_TOKEN_SENTINEL};
use crate::onchainos::{extract_tx_hash, resolve_wallet, wallet_contract_call};
use crate::rpc::{erc20_allowance, erc20_balance, fmt_token_amount, native_balance, wait_for_tx};

#[derive(Args)]
pub struct BridgeArgs {
    /// Source chain (id or key)
    #[arg(long)]
    pub from_chain: String,
    /// Destination chain (id or key)
    #[arg(long)]
    pub to_chain: String,
    /// Source token (symbol or 0x address; "ETH"/"BNB"/etc. for native)
    #[arg(long)]
    pub from_token: String,
    /// Destination token (symbol or 0x address)
    #[arg(long)]
    pub to_token: String,
    /// Human-readable amount (e.g. 100 = 100 USDC)
    #[arg(long, allow_hyphen_values = true)]
    pub amount: String,
    /// Receiver address (defaults to sender)
    #[arg(long)]
    pub to_address: Option<String>,
    /// Slippage as a percent (default 0.5 = 0.5%)
    #[arg(long, default_value = "0.5")]
    pub slippage_pct: f64,
    /// Route preference (default FASTEST)
    #[arg(long, default_value = "FASTEST")]
    pub order: String,
    /// Bridges to exclude (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub deny_bridges: Vec<String>,
    /// Dry run — fetches quote, validates, prints calldata, but does not sign or submit
    #[arg(long)]
    pub dry_run: bool,
    /// Required to actually submit (without it, prints a preview and stops)
    #[arg(long)]
    pub confirm: bool,
    /// Approve confirmation timeout (seconds, default 180)
    #[arg(long, default_value = "180")]
    pub approve_timeout_secs: u64,
}

pub async fn run(args: BridgeArgs) -> anyhow::Result<()> {
    // ── 1. Resolve chains ─────────────────────────────────────────────────────
    let from_chain = match parse_chain(&args.from_chain) {
        Some(c) => c,
        None => return print_err(
            &format!("Unsupported source chain '{}'", args.from_chain),
            "UNSUPPORTED_CHAIN",
            &format!("Use one of: {}", supported_chains_help()),
        ),
    };
    let to_chain = match parse_chain(&args.to_chain) {
        Some(c) => c,
        None => return print_err(
            &format!("Unsupported destination chain '{}'", args.to_chain),
            "UNSUPPORTED_CHAIN",
            &format!("Use one of: {}", supported_chains_help()),
        ),
    };

    let order = args.order.to_uppercase();
    if order != "FASTEST" && order != "CHEAPEST" {
        return print_err(
            &format!("--order must be FASTEST or CHEAPEST (got '{}')", args.order),
            "INVALID_ARGUMENT",
            "Use --order FASTEST or --order CHEAPEST",
        );
    }
    if args.slippage_pct < 0.0 || args.slippage_pct > 50.0 {
        return print_err(
            &format!("Slippage {}% out of range (0–50)", args.slippage_pct),
            "INVALID_ARGUMENT",
            "Pass slippage in percent (0.5 = 0.5%, not 0.005).",
        );
    }

    // ── 2. Resolve wallet on source chain ─────────────────────────────────────
    let from_addr = match resolve_wallet(from_chain.id) {
        Ok(a) => a,
        Err(e) => return print_err(
            &format!("Could not resolve wallet on chain {}: {:#}", from_chain.id, e),
            "WALLET_NOT_FOUND",
            "Run `onchainos wallet addresses` to verify login.",
        ),
    };

    // ── 3. Resolve tokens ─────────────────────────────────────────────────────
    let (from_token_addr, from_token_decimals, from_token_symbol) =
        match resolve_token(from_chain.id, &args.from_token, from_chain.native_symbol).await {
            Ok(t) => t,
            Err(e) => return print_err(
                &format!("from_token '{}' on chain {}: {:#}", args.from_token, from_chain.key, e),
                "TOKEN_NOT_FOUND",
                "Pass the 0x… contract address or verify the symbol via `tokens --chain X --symbol Y`.",
            ),
        };
    let (to_token_addr, _to_token_decimals, _to_token_symbol) =
        match resolve_token(to_chain.id, &args.to_token, to_chain.native_symbol).await {
            Ok(t) => t,
            Err(e) => return print_err(
                &format!("to_token '{}' on chain {}: {:#}", args.to_token, to_chain.key, e),
                "TOKEN_NOT_FOUND",
                "Pass the 0x… contract address or verify the symbol via `tokens --chain X --symbol Y`.",
            ),
        };

    // ── 4. Convert amount + assemble quote params ─────────────────────────────
    let amount_raw = match human_to_atomic(&args.amount, from_token_decimals) {
        Ok(s) => s,
        Err(e) => return print_err(
            &format!("Invalid amount '{}': {}", args.amount, e),
            "INVALID_ARGUMENT",
            "Pass a positive number, e.g. --amount 100 or --amount 0.001",
        ),
    };
    let amount_atomic: u128 = amount_raw.parse().unwrap_or(0);
    let slippage_dec = args.slippage_pct / 100.0;
    let deny: Vec<&str> = args.deny_bridges.iter().map(|s| s.as_str()).collect();

    // ── 5. Get the quote first (we need its gasCosts for the gas check) ──────
    let qp = QuoteParams {
        from_chain: from_chain.id,
        to_chain: to_chain.id,
        from_token: &from_token_addr,
        to_token: &to_token_addr,
        from_address: &from_addr,
        to_address: args.to_address.as_deref(),
        from_amount: &amount_raw,
        slippage: Some(slippage_dec),
        order: Some(&order),
        deny_bridges: deny,
        integrator: Some("lifi-plugin"),
    };

    let quote = match api::get_quote(&qp).await {
        Ok(v) => v,
        Err(e) => {
            let msg = format!("{:#}", e);
            let (code, suggestion) = classify_quote_error(&msg);
            return print_err(&msg, code, suggestion);
        }
    };

    // ── 6. Pre-flight balance check (EVM-001 source-token + new native-gas check) ─
    // Always read native balance — needed for gas check even when from-token is ERC-20.
    let native_bal = match native_balance(&from_addr, from_chain.rpc).await {
        Ok(v) => v,
        Err(e) => return print_err(
            &format!("Failed to read native balance on {}: {:#}", from_chain.key, e),
            "RPC_ERROR",
            "Public RPC may be limited. Retry in a few seconds.",
        ),
    };
    let from_token_bal: u128 = if is_native_token(&from_token_addr) {
        native_bal
    } else {
        match erc20_balance(&from_token_addr, &from_addr, from_chain.rpc).await {
            Ok(v) => v,
            Err(e) => return print_err(
                &format!("Failed to read {} balance on {}: {:#}", from_token_symbol, from_chain.key, e),
                "RPC_ERROR",
                "Public RPC may be limited. Retry in a few seconds.",
            ),
        }
    };

    // Source-token balance check (EVM-001).
    if from_token_bal < amount_atomic {
        return print_err(
            &format!(
                "Insufficient {} on {}: need {} (raw {}), have {} (raw {})",
                from_token_symbol, from_chain.key,
                fmt_token_amount(amount_atomic, from_token_decimals), amount_atomic,
                fmt_token_amount(from_token_bal, from_token_decimals), from_token_bal
            ),
            "INSUFFICIENT_BALANCE",
            "Top up the source token on the source chain, or reduce --amount.",
        );
    }

    // Native gas balance check. Sum estimate.gasCosts[].amount; if from-token
    // is native, the bridge amount is also debited from native balance, so we
    // require: native_bal >= amount + gas. Otherwise just: native_bal >= gas.
    let gas_total_wei = sum_gas_costs(&quote);
    let native_required = if is_native_token(&from_token_addr) {
        amount_atomic.saturating_add(gas_total_wei)
    } else {
        gas_total_wei
    };
    if native_bal < native_required {
        let shortfall = native_required - native_bal;
        return print_err(
            &format!(
                "Insufficient native gas on {}: need {} {} (≈ {} wei), have {} {} (gas alone: {} wei)",
                from_chain.key,
                fmt_token_amount(native_required, 18), from_chain.native_symbol, native_required,
                fmt_token_amount(native_bal, 18), from_chain.native_symbol,
                gas_total_wei,
            ),
            "INSUFFICIENT_GAS",
            &format!(
                "Top up native {} on {} by ≈{} {} (or reduce --amount if you're bridging native).",
                from_chain.native_symbol, from_chain.key,
                fmt_token_amount(shortfall, 18), from_chain.native_symbol,
            ),
        );
    }

    let approval_addr = quote["estimate"]["approvalAddress"]
        .as_str()
        .map(|s| s.to_string());
    let tx_req = &quote["transactionRequest"];
    let router_to = match tx_req["to"].as_str() {
        Some(s) => s.to_string(),
        None => return print_err(
            "Quote response missing transactionRequest.to",
            "BAD_QUOTE_RESPONSE",
            "Retry. If persistent, this LI.FI route may be temporarily unavailable.",
        ),
    };
    let calldata = match tx_req["data"].as_str() {
        Some(s) => s.to_string(),
        None => return print_err(
            "Quote response missing transactionRequest.data",
            "BAD_QUOTE_RESPONSE",
            "Retry, or pass --order CHEAPEST to try a different route.",
        ),
    };
    let value_hex = tx_req["value"].as_str().unwrap_or("0x0");
    let value_wei = parse_hex_u128(value_hex);

    let to_amount_min_atomic: u128 = quote["estimate"]["toAmountMin"]
        .as_str().unwrap_or("0").parse().unwrap_or(0);

    // ── 7. Preview block ──────────────────────────────────────────────────────
    // Always wrap with ok:true so stdout is a single parseable JSON object
    // (knowledge base GEN-001: structured JSON envelope on every code path).
    let stage = if args.dry_run { "dry_run" } else if args.confirm { "submit" } else { "preview" };
    let exec_secs = quote["estimate"]["executionDuration"].as_u64().unwrap_or(0);
    let tool_name = quote.get("tool").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let reliability = assess_reliability(&tool_name, exec_secs, amount_atomic, from_token_decimals);

    let preview = json!({
        "ok": true,
        "stage": stage,
        "submitted": false,
        "preview": {
            "tool": tool_name,
            "type": quote.get("type").cloned().unwrap_or(Value::Null),
            "from": {
                "chain": from_chain.key, "chain_id": from_chain.id,
                "token": from_token_symbol,
                "amount": fmt_token_amount(amount_atomic, from_token_decimals),
                "amount_raw": amount_atomic.to_string(),
                "wallet": from_addr,
                "token_balance": fmt_token_amount(from_token_bal, from_token_decimals),
                "token_balance_raw": from_token_bal.to_string(),
                "native_balance": fmt_token_amount(native_bal, 18),
                "native_balance_raw": native_bal.to_string(),
            },
            "to": {
                "chain": to_chain.key, "chain_id": to_chain.id,
                "amount_min_raw": to_amount_min_atomic.to_string(),
                "wallet": args.to_address.clone().unwrap_or(quote["action"]["toAddress"].as_str().unwrap_or("").to_string()),
            },
            "approval_address": approval_addr.clone().unwrap_or_default(),
            "router_to": router_to,
            "value_wei": value_wei.to_string(),
            "execution_duration_seconds": exec_secs,
            "slippage_pct": args.slippage_pct,
            "is_native_input": is_native_token(&from_token_addr),
            "gas": {
                "estimate_wei": gas_total_wei.to_string(),
                "estimate_native": fmt_token_amount(gas_total_wei, 18),
                "native_required_total_wei": native_required.to_string(),
                "native_required_total": fmt_token_amount(native_required, 18),
            },
            "reliability": reliability,
        }
    });
    println!("{}", serde_json::to_string_pretty(&preview)?);

    if args.dry_run {
        eprintln!("[DRY RUN] Calldata fetched, balance verified. Not signing or submitting.");
        return Ok(());
    }
    if !args.confirm {
        eprintln!("[PREVIEW] Add --confirm to sign and submit the bridge transaction.");
        return Ok(());
    }

    // ── 8. Approve (if ERC-20 and allowance < amount). EVM-005, EVM-006. ─────
    if !is_native_token(&from_token_addr) {
        let spender = match approval_addr.as_deref() {
            Some(s) => s.to_string(),
            None => return print_err(
                "ERC-20 approval required but quote.estimate.approvalAddress missing",
                "BAD_QUOTE_RESPONSE",
                "Retry quote, or pick another route.",
            ),
        };
        let allowance = match erc20_allowance(&from_token_addr, &from_addr, &spender, from_chain.rpc).await {
            Ok(a) => a,
            Err(e) => return print_err(
                &format!("Failed to read allowance: {:#}", e),
                "RPC_ERROR",
                "Public RPC may be limited. Retry in a few seconds.",
            ),
        };
        if allowance < amount_atomic {
            // 0xa9059cbb is transfer; we want approve(spender, MAX): selector 0x095ea7b3
            let approve_data = build_approve_max(&spender);
            eprintln!(
                "[bridge] Approving {} for {} (current allowance {} < {} required)…",
                from_token_symbol, spender, allowance, amount_atomic
            );
            let result = match wallet_contract_call(from_chain.id, &from_token_addr, &approve_data, None, false) {
                Ok(r) => r,
                Err(e) => return print_err(
                    &format!("Approve submission failed: {:#}", e),
                    "APPROVE_FAILED",
                    "Check onchainos status and gas balance on the source chain.",
                ),
            };
            let approve_hash = match extract_tx_hash(&result) {
                Some(h) => h,
                None => return print_err(
                    "Approve submitted but tx hash not returned by onchainos",
                    "APPROVE_HASH_MISSING",
                    "Inspect raw onchainos output for txHash; retry if not visible.",
                ),
            };
            eprintln!("[bridge] Approve tx: {} — waiting for confirmation…", approve_hash);
            if let Err(e) = wait_for_tx(&approve_hash, from_chain.rpc, args.approve_timeout_secs).await {
                return print_err(
                    &format!("Approve tx did not confirm: {:#}", e),
                    "APPROVE_NOT_CONFIRMED",
                    "Bump --approve-timeout-secs or check explorer for the approve tx status.",
                );
            }
            eprintln!("[bridge] Approve confirmed.");
        } else {
            eprintln!("[bridge] Existing allowance {} >= required {}; skipping approve.", allowance, amount_atomic);
        }
    }

    // ── 9. Submit the bridge tx via onchainos ─────────────────────────────────
    let submit_value = if is_native_token(&from_token_addr) { Some(value_wei) } else { None };
    let result = match wallet_contract_call(from_chain.id, &router_to, &calldata, submit_value, false) {
        Ok(r) => r,
        Err(e) => return print_err(
            &format!("Bridge submission failed: {:#}", e),
            "BRIDGE_SUBMIT_FAILED",
            "Inspect onchainos output. Common causes: insufficient gas, RPC issue, slippage tightened.",
        ),
    };
    let tx_hash = extract_tx_hash(&result);

    println!("{}", serde_json::to_string_pretty(&json!({
        "ok": true,
        "action": "bridge",
        "from_chain": from_chain.key,
        "to_chain": to_chain.key,
        "from_token": from_token_symbol,
        "amount": fmt_token_amount(amount_atomic, from_token_decimals),
        "amount_raw": amount_atomic.to_string(),
        "tool": quote.get("tool").cloned().unwrap_or(Value::Null),
        "tx_hash": tx_hash,
        "execution_duration_seconds": quote["estimate"]["executionDuration"].as_u64(),
        "tip": "Run `lifi-plugin status --tx-hash <h> --from-chain <X> --to-chain <Y>` to track the cross-chain leg.",
    }))?);
    Ok(())
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn print_err(msg: &str, code: &str, suggestion: &str) -> anyhow::Result<()> {
    println!("{}", super::error_response(msg, code, suggestion));
    Ok(())
}

/// Sum estimate.gasCosts[].amount across all gas-cost entries (SEND + APPROVE if present).
/// Returns total wei the caller must hold in native gas to cover the bridge tx(s).
fn sum_gas_costs(quote: &Value) -> u128 {
    quote["estimate"]["gasCosts"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|g| g.get("amount").and_then(|a| a.as_str()))
                .filter_map(|s| s.parse::<u128>().ok())
                .sum()
        })
        .unwrap_or(0)
}

/// Assess revert risk for the chosen LI.FI tool at the given amount, and surface
/// a warning the user can act on. Returns Value::Null when no concern detected.
///
/// Heuristics (from real-world failure modes observed):
///   - `mayan` and `near` use signed off-chain solver quotes that expire fast.
///     Submission may revert if onchainos broadcast latency exceeds the quote
///     validity window. Common at small amounts where these are the only routes.
///   - executionDuration > 30s usually means a slow / async settlement tool.
fn assess_reliability(tool: &str, exec_secs: u64, _amount_atomic: u128, _decimals: u32) -> Value {
    let lc = tool.to_lowercase();
    let solver_quote = matches!(lc.as_str(), "mayan" | "near" | "relayer");

    if solver_quote {
        return json!({
            "level": "WARN",
            "tool": tool,
            "concern": "solver_quote_latency",
            "message": format!(
                "Tool '{}' uses time-sensitive signed quotes from off-chain solvers. The signed quote may expire between fetch and broadcast, causing on-chain revert with no informative reason. Observed in production at small (<$25) amounts.",
                tool
            ),
            "suggestions": [
                "Re-run with --deny-bridges mayan,near — typically falls back to `across` or `relaydepository`, both of which submit deterministic calldata and confirm in ~3s. (Recommended.)",
                "If --deny-bridges returns NO_ROUTE_AVAILABLE, try a different destination chain. `lifi-plugin routes --from-chain X --to-chain Y --limit 8` lists alternatives.",
                "Increase --amount — at ≥$25 USD-equivalent more bridges become available so the default picker is less likely to land on a solver-quote tool.",
            ],
        });
    }

    if exec_secs > 30 {
        return json!({
            "level": "INFO",
            "tool": tool,
            "concern": "long_execution_duration",
            "message": format!("Tool '{}' has an estimated execution duration of {}s. This is normal for some bridges (e.g. Stargate, async settlement), but plan accordingly.", tool, exec_secs),
            "suggestions": ["After --confirm, use `lifi-plugin status --tx-hash <h>` to track until status: DONE."],
        });
    }

    Value::Null
}

fn human_to_atomic(s: &str, decimals: u32) -> Result<String, String> {
    let f: f64 = s.parse().map_err(|_| "not a number".to_string())?;
    if f <= 0.0 || !f.is_finite() {
        return Err("must be a positive finite number".to_string());
    }
    let scaled = f * 10f64.powi(decimals as i32);
    if scaled > u128::MAX as f64 {
        return Err("amount exceeds u128".to_string());
    }
    let atomic = scaled.round() as u128;
    if atomic == 0 {
        return Err(format!("amount too small for {} decimals", decimals));
    }
    Ok(atomic.to_string())
}

async fn resolve_token(
    chain_id: u64,
    user_input: &str,
    native_symbol: &str,
) -> anyhow::Result<(String, u32, String)> {
    let trimmed = user_input.trim();
    let upper = trimmed.to_uppercase();
    if is_native_token(trimmed)
        || upper == native_symbol
        || upper == "ETH" || upper == "BNB" || upper == "MATIC" || upper == "POL"
        || upper == "NATIVE"
    {
        return Ok((NATIVE_TOKEN_SENTINEL.to_string(), 18, native_symbol.to_string()));
    }
    let info = api::get_token(chain_id, trimmed).await?;
    let address = info["address"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("LI.FI did not return an address for '{}'", trimmed))?
        .to_string();
    let decimals = info["decimals"]
        .as_u64()
        .ok_or_else(|| anyhow::anyhow!("LI.FI did not return decimals for '{}'", trimmed))?
        as u32;
    let symbol = info["symbol"].as_str().unwrap_or(trimmed).to_string();
    Ok((address, decimals, symbol))
}

fn classify_quote_error(msg: &str) -> (&'static str, &'static str) {
    if msg.contains("404") || msg.contains("No quote available") || msg.contains("No available quote") {
        ("NO_ROUTE_AVAILABLE", "No route exists for this pair. Try a different token or smaller amount.")
    } else if msg.contains("400") || msg.contains("Invalid") {
        ("INVALID_QUOTE_REQUEST", "Quote params rejected. Verify chain/token/amount.")
    } else if msg.contains("INSUFFICIENT_LIQUIDITY") {
        ("INSUFFICIENT_LIQUIDITY", "Pool depth is too thin for this size. Try a smaller amount.")
    } else {
        ("API_ERROR", "LI.FI quote API failed. Retry or check connectivity.")
    }
}

fn parse_hex_u128(s: &str) -> u128 {
    let t = s.trim();
    if t.is_empty() || t == "0" || t == "0x0" {
        return 0;
    }
    if let Some(h) = t.strip_prefix("0x") {
        u128::from_str_radix(h, 16).unwrap_or(0)
    } else {
        t.parse().unwrap_or(0)
    }
}

/// Build calldata for ERC20.approve(spender, type(uint256).max).
/// Selector 0x095ea7b3.
fn build_approve_max(spender: &str) -> String {
    let s = spender.trim_start_matches("0x");
    format!("0x095ea7b3{:0>64}{}", s, "f".repeat(64))
}
