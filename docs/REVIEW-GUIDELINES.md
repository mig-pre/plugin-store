# Review Guidelines

This document describes the review process that every plugin submission to the OKX Plugin Store must pass before it can be merged. Understanding these standards will help you submit clean plugins and avoid delays.

---

## 1. Review Process Overview

Every pull request targeting the `skills/` directory goes through up to five sequential phases:

```
Submit PR to skills/<plugin-name>/
    |
    v
+-------------------------------+
| Phase 1: Structure Validation |  plugin-lint.yml  ~30 seconds
+-------------------------------+
    |
    v
+-------------------------------+
| Phase 2: Build Verification   |  plugin-build.yml  1-5 minutes (if applicable)
+-------------------------------+
    |
    v
+-------------------------------+
| Phase 3: AI Code Review       |  plugin-ai-review.yml  2-5 minutes (advisory)
+-------------------------------+
    |
    v
+-------------------------------+
| Phase 4: Summary + Pre-flight |  plugin-summary.yml  1-2 minutes (maintainer-gated)
+-------------------------------+
    |
    v
+-------------------------------+
| Phase 5: Human Review         |  skill-review.yml + CODEOWNERS  1-3 business days
+-------------------------------+
    |
    v
 Approved  or  Changes Requested
```

A failure in Phase 1 or Phase 2 blocks progression. Phase 3 is **advisory only** -- it does not block merge but provides a detailed security report for human reviewers. Phase 4 requires maintainer approval via the `summary-generation` GitHub environment gate. Phase 5 is the final human decision.

Additionally, the `skill-review.yml` workflow runs independently on every PR and push to `main`, performing static checks and a lightweight AI review of official plugins.

---

## 2. Phase 1: Structure Validation (`plugin-lint.yml`)

An automated linter validates structural correctness, safety defaults, and metadata consistency. This workflow uses `pull_request_target` to support PRs from forks with write permissions for comments and labels.

### Pre-Lint Checks

Before lint rules run, the workflow enforces three structural constraints:

| Check | Behavior |
|-------|----------|
| **Single plugin per PR** | PR must only modify files within one `skills/<name>/` directory. Modifying multiple plugins fails the check. |
| **No files outside `skills/`** | PR must not modify any file outside the `skills/` directory tree. |
| **Name uniqueness** | For new plugins, the name must not already exist in `registry.json`. |

### OKX Org Membership Detection

The workflow detects whether the PR author is an OKX org member (by checking if the source repo is under `okx/` or if the author is a public org member). This controls whether reserved prefixes like `okx-` and `official-` are permitted in plugin names.

### Lint Rules

| Category | Check | Severity |
|----------|-------|----------|
| **Structure** | `plugin.yaml` exists at plugin root | Error |
| **Structure** | `SKILL.md` exists (for skill-type plugins) | Error |
| **Structure** | `plugin.yaml` contains valid YAML | Error |
| **Version Consistency** | Version in `plugin.yaml` matches version declared in `SKILL.md` frontmatter | Error |
| **Safety Defaults** | `PAUSED=True` is set (plugins must not auto-start) | Error |
| **Safety Defaults** | `PAPER_TRADE=True` is set (live trading must be opt-in) | Error |
| **Safety Defaults** | `DRY_RUN=True` is set (destructive actions must be opt-in) | Error |
| **Python Validation** | All `.py` files pass syntax check (`ast.parse`) | Error |
| **URL Checks** | All URLs referenced in SKILL.md return HTTP 2xx (404 = warning) | Warning |
| **Category** | Category is one of: `trading`, `defi`, `game`, `prediction`, `data_tools`, `dev_tools`, `others` | Error |
| **License** | License field contains a valid SPDX identifier (e.g., `MIT`, `Apache-2.0`) | Error |
| **Name Uniqueness** | New plugin name does not collide with existing registry entries | Error |

### Labels

The workflow automatically applies PR labels based on results:

| Condition | Label Added | Label Removed |
|-----------|-------------|---------------|
| New plugin submission | `new-plugin` | -- |
| Update to existing plugin | `plugin-update` | -- |
| Lint passes | `structure-validated` | `needs-fix` |
| Lint fails | `needs-fix` | `structure-validated` |

### Severity Levels

- **Error** -- Blocks merge. The PR cannot proceed until the issue is resolved.
- **Warning** -- Advisory. Flagged for awareness but does not block merge.

---

## 3. Phase 2: Build Verification (`plugin-build.yml`)

If your `plugin.yaml` includes a `build` section, the CI system clones your source code and compiles it. This follows a Homebrew-like model: source lives in your external repo, and plugin-store CI builds and distributes the artifact.

### Source Code Model

The build system supports two source modes:

| Mode | Trigger | Behavior |
|------|---------|----------|
| **External** | `build.source_repo` is set in `plugin.yaml` | Clones the external repo at `build.source_commit` (pinned SHA) |
| **Local** | No `source_repo` set | Uses source code directly from the `skills/<name>/` directory |

### Supported Languages

| Language | Build Tool | Trigger | Security Audit |
|----------|-----------|---------|----------------|
| Rust | `cargo build --release` | `build.lang: rust` | `cargo audit` |
| Go | `go build -ldflags="-s -w"` | `build.lang: go` | `govulncheck` |
| TypeScript | `bun build --compile` | `build.lang: typescript` | -- |
| Node.js | `bun build --compile` | `build.lang: node` | -- |
| Python | `pip install` + entry point verification | `build.lang: python` | `pip-audit` |

### Build Pipeline Steps

For each supported language, the build job:

1. **Prepares source** -- clones external repo at pinned commit or copies local source
2. **Fetches dependencies** -- `cargo fetch`, `go mod download`, `bun install`, or `pip install`
3. **Runs security audit** -- language-specific vulnerability scanner (where available)
4. **Compiles** -- produces binary artifact
5. **Verifies artifact** -- checks that the expected binary exists, records size and SHA256
6. **Uploads artifact** -- stores build output as a GitHub Actions artifact

### Build Report

A combined report is posted as a PR comment showing:
- Plugin name, language, and source repo with commit SHA
- Build result (pass/fail/skipped)
- Link to source commit for auditability

### When Build Is Skipped

If your plugin has no `build` section in `plugin.yaml` (e.g., it is a pure SKILL.md plugin with no compiled binary), Phase 2 is skipped entirely.

---

## 4. Phase 3: AI Code Review (`plugin-ai-review.yml`)

An AI reviewer performs a structured audit across nine dimensions, producing a detailed report posted as a PR comment. This phase is **advisory** -- it never fails the PR check.

### Environment Gate

This workflow runs under the `ai-review` GitHub environment. The Claude API call is made using `ANTHROPIC_API_KEY` (or `OPENROUTER_API_KEY` if configured). The workflow automatically selects the best available Claude model (preferring Opus, falling back to Sonnet).

### Live onchainos Context

Before reviewing, the workflow clones the latest onchainos source code (`okx/onchainos-skills` main branch) to use as review context. This ensures the AI knows exactly which CLI commands exist and can verify whether plugins correctly reference real onchainos capabilities.

### Nine Audit Dimensions

| # | Dimension | What Is Evaluated |
|---|-----------|-------------------|
| 1 | **Plugin Overview** | Name, version, category, author, license, risk level, and a plain-language summary of what the plugin does. |
| 2 | **Architecture Analysis** | Component structure (skill/binary), SKILL.md organization, data flow, and external dependencies. |
| 3 | **Permission Analysis** | Inferred permissions: onchainos commands used, wallet operations detected, external APIs/URLs called, and chains operated on. |
| 4 | **OnchainOS Compliance** | Whether all on-chain write operations (signing, broadcasting, swaps, approvals, transfers) use the onchainos CLI rather than self-implementing with raw libraries. This is the single most important check. |
| 5 | **Security Assessment** | Application of static security rules, LLM semantic judges, and toxic flow detection (see below). Includes prompt injection scanning, dangerous operation checks, and data exfiltration risk analysis. |
| 6 | **Source Code Review** | Language and build config, dependency audit, code safety checks (hardcoded secrets, undeclared network requests, filesystem access, dynamic code execution, unsafe blocks). Only applies to plugins with source code. |
| 7 | **Code Quality** | Scored on five sub-dimensions: Completeness (25 pts), Clarity (25 pts), Security Awareness (25 pts), Skill Routing (15 pts), and Formatting (10 pts). |
| 8 | **Recommendations** | Prioritized list of actionable improvements. |
| 9 | **Summary and Score** | One-line verdict, merge recommendation, and an overall quality score from 0 to 100. |

### Three-Layer Security Scanning

The security assessment in Dimension 5 uses three complementary detection layers, defined in `.github/security-rules/`.

#### Layer 1: Static Rules (28 rules in `static-rules.md`)

Pattern-based scanning across four severity levels. The scanner checks for known dangerous patterns without requiring semantic understanding.

| Severity | Count | What Is Detected |
|----------|-------|------------------|
| **Critical** (C01-C09) | 9 | Command injection (pipe to shell), prompt injection keywords, base64/unicode obfuscation, credential exfiltration via environment variables or command substitution, password-protected archive downloads, pseudo-system tag injection, hidden instructions in HTML comments, backtick injection with sensitive paths. |
| **High** (H01-H09) | 9 | Hardcoded secrets (API keys, private keys, mnemonics), instructions to output credentials, persistence mechanisms (cron, launchctl, systemd), access to sensitive file paths (~/.ssh, ~/.aws, ~/.kube), direct financial/on-chain API operations, system file modification, plaintext credential storage in .env files, credential solicitation in chat, signed transaction data in CLI parameters. |
| **Medium** (M01-M08) | 8 | Unpinned package installations, unverifiable runtime dependencies, third-party content fetching without boundary markers, resource exhaustion patterns (fork bombs, infinite loops), dynamic package installation via eval/exec, skill chaining without version pinning, missing untrusted-data boundary declarations, external data field passthrough without isolation. |
| **Low** (L01-L02) | 2 | Agent capability discovery/enumeration attempts, undeclared network communication (raw IPs, DNS lookups, netcat). |

**Judgment logic:**
- Any Critical finding = **FAIL** (blocks merge)
- Any High or Medium finding (without Critical) = **WARN** (flagged for human review)
- Only Low/Info or no findings = **PASS**

**Phase 3.5 re-adjudication**: Some rules have context-dependent severity. For example, C01 (curl|sh) is downgraded from CRITICAL to MEDIUM if found in README.md or install scripts rather than SKILL.md. H05 (financial API operations) is INFO by default and only escalates when combined with other findings.

#### Layer 2: LLM Semantic Judges (6 judges in `llm-judges.md`)

AI-powered semantic analysis that detects threats beyond pattern matching:

| Judge | Severity | What It Detects |
|-------|----------|-----------------|
| **L-PINJ: Prompt Injection** | Critical | Hidden instructions that hijack agent behavior, including instruction overrides, pseudo-system tags, encoded payloads, jailbreak attempts, and CLI parameter injection via unsanitized user input. |
| **L-MALI: Malicious Intent** | Critical | Discrepancy between a plugin's stated purpose and its actual behavior -- e.g., a "wallet tracker" that secretly uploads private keys. |
| **L-MEMA: Memory Poisoning** | High | Attempts to write to agent memory files (MEMORY.md, SOUL.md) to plant cross-session backdoors that survive restarts. |
| **L-IINJ: External Request Notice** | Info/Medium | External API or CLI calls. Rated Info if the plugin declares an untrusted-data boundary; Medium if it does not. |
| **L-AEXE: Autonomous Execution Risk** | Info | Operations that could be executed without explicit user confirmation (vague authorization words like "proceed", "handle", "automatically" without confirmation gates). |
| **L-FINA: Financial Scope Assessment** | Info to Critical | Evaluates the financial operation scope: read-only queries (exempt), confirmed writes (Info), unconfirmed writes (High), fully autonomous fund transfers (Critical). |

Results with confidence below 0.7 are automatically discarded.

#### Layer 3: Toxic Flow Detection (5 attack chains in `toxic-flows.md`)

Combinations of individually lower-severity findings that together form a complete attack chain:

| Flow | Trigger Combination | Severity | Attack Pattern |
|------|---------------------|----------|----------------|
| **TF001** | Sensitive path access + credential exfiltration or undeclared network | Critical | Read credentials from ~/.ssh or ~/.aws, then exfiltrate via HTTP/DNS/netcat. Complete credential theft chain. |
| **TF002** | Prompt injection + persistence mechanism | Critical | Jailbreak the agent's safety guardrails, then register a persistent service (cron/launchctl) that survives reboots. |
| **TF004** | Unverifiable dependency + malicious intent detected | High | Malicious plugin installs additional unverified packages whose postinstall hooks execute attack payloads. |
| **TF005** | Command injection (curl pipe sh) + financial API access | Critical | Remote script (replaceable at any time) combined with financial operations enables unauthorized fund transfers. |
| **TF006** | Missing data boundary (M07/M08) + financial operations (H05) | High | External data (token names, swap routes) enters agent context without isolation; attacker injects instructions via on-chain fields to manipulate transaction parameters. |

### Watchdog

After the AI review completes, a watchdog step verifies that the report comment was actually posted to the PR. If no report is found (e.g., due to API rate limiting), the watchdog posts a warning comment and marks the workflow as failed so maintainers are alerted.

### Quality Score Interpretation

| Score | Meaning |
|-------|---------|
| 80-100 | Ready to merge. No significant issues found. |
| 60-79 | Minor issues identified. Likely approved after targeted fixes. |
| Below 60 | Significant concerns. Substantial changes required before re-review. |

---

## 5. Phase 4: Summary + Pre-flight (`plugin-summary.yml`)

This phase generates user-facing summaries and injects pre-flight safety checks. It requires maintainer approval via the `summary-generation` GitHub environment gate.

### What It Does

1. **Resolves SKILL.md** -- finds the SKILL.md from local submission or external repo (following `components.skill.repo` in plugin.yaml).
2. **Generates SUMMARY.md** -- a Claude-generated plain-language description of the plugin for the registry listing.
3. **Generates SKILL_SUMMARY.md** -- a condensed version of the skill's capabilities for quick reference.
4. **Injects pre-flight checks** -- runs `inject-preflight.py` to add safety pre-flight sections to the SKILL.md if missing.
5. **Pushes changes** -- commits the generated files and pre-flight patches back to the PR branch.

### Generated Files

| File | Purpose |
|------|---------|
| `SUMMARY.md` | Plain-language plugin description for the store listing |
| `SKILL_SUMMARY.md` | Condensed capability summary |
| Pre-flight patches in `SKILL.md` | Safety checks injected before dangerous operations |

---

## 6. Phase 5: Human Review

After automated checks pass, human reviewers examine the submission. Review assignment is controlled by `CODEOWNERS`:

| Path | Reviewer Team |
|------|---------------|
| `skills/` (all plugins) | `@okx/plugin-store-reviewers` |
| `skills/uniswap-*/`, `skills/polymarket-*/` | `@okx/plugin-store-core` |
| `skills/plugin-store/` | `@okx/plugin-store-core` |

### Review Focus by Risk Level

| Plugin Risk Level | Review Depth | Reviewer Count |
|-------------------|-------------|----------------|
| Low (read-only, data display) | Standard review of SKILL.md and metadata | 1 reviewer |
| Medium (writes data, calls external APIs) | Detailed review including data flow analysis | 1 reviewer |
| High/Advanced (financial operations, on-chain writes) | Full security audit of all code and instructions | 2 reviewers required |

### What Human Reviewers Focus On

- Accuracy of the AI review report (confirming or overriding AI findings)
- Business logic correctness that AI may miss
- User experience and documentation quality
- Edge cases in financial operations
- Consistency with existing Plugin Store standards

### SLA

Human review is completed within **1 to 3 business days** of passing Phase 3. Complex or high-risk plugins may take longer if additional reviewers are needed.

---

## 7. Absolute Prohibitions (10 Red Lines)

The following will result in **immediate rejection** regardless of any other factors. These are non-negotiable.

| # | Prohibition | Why |
|---|------------|-----|
| 1 | **Hardcoded private keys, mnemonics, or API secrets** | Credentials in source code are permanently exposed in version history. |
| 2 | **Command injection (`curl \| sh` with remote URLs)** | Remote scripts can be replaced at any time, enabling arbitrary code execution. |
| 3 | **Prompt injection attempts** | Instructions that override agent safety guardrails compromise all users. |
| 4 | **Credential exfiltration** | Any mechanism that sends local credentials (env vars, files) to external servers. |
| 5 | **Obfuscated code (base64 payloads, unicode tricks)** | Code that cannot be read by reviewers cannot be trusted. |
| 6 | **Persistence mechanisms (cron, launchctl, systemd)** | Background services survive plugin uninstall and can act as long-term backdoors. |
| 7 | **Accessing sensitive files (~/.ssh, ~/.aws, ~/.kube, ~/.gnupg)** | No plugin has a legitimate reason to read SSH keys or cloud credentials. |
| 8 | **Direct financial operations bypassing OnchainOS without declaration** | All on-chain write operations must go through the onchainos CLI. Self-implementing wallet signing, transaction broadcasting, or swap execution is forbidden. |
| 9 | **Supply chain attacks (unpinned dependencies + dynamic install)** | Runtime installation of unversioned packages opens an ever-present poisoning window. |
| 10 | **Memory poisoning attempts** | Writing to agent memory files (MEMORY.md, SOUL.md) to plant persistent cross-session instructions. |

---

## 8. Pre-Submission Checklist

Copy this checklist into your PR description before submitting:

```markdown
## Pre-Submission Checklist

- [ ] `plugin.yaml` exists and contains valid YAML
- [ ] `SKILL.md` exists with correct version matching `plugin.yaml`
- [ ] `.claude-plugin/plugin.json` exists (if applicable)
- [ ] Category is one of: trading, defi, game, prediction, data_tools, dev_tools, others
- [ ] License field contains a valid SPDX identifier
- [ ] Safety defaults set: PAUSED=True, PAPER_TRADE=True, DRY_RUN=True
- [ ] No hardcoded secrets, private keys, or mnemonics in any file
- [ ] No `curl | sh` or `wget | sh` patterns
- [ ] No obfuscated code (base64 payloads, unicode encoding)
- [ ] No access to sensitive paths (~/.ssh, ~/.aws, ~/.kube)
- [ ] All on-chain write operations use onchainos CLI (no raw ethers.js, web3.py, etc.)
- [ ] All external URLs are reachable
- [ ] All package dependencies are version-pinned
- [ ] External data has untrusted-data boundary declaration in SKILL.md
- [ ] Financial operations include explicit user confirmation steps
- [ ] Python files pass syntax check
- [ ] Build succeeds on all target platforms (if applicable)
- [ ] PR only modifies files within one `skills/<name>/` directory
```

---

## 9. Appeals Process

If you believe a review decision is incorrect:

1. **Comment on the PR** with a clear explanation of why you disagree with the finding. Include evidence (code references, documentation links) supporting your case.
2. **A reviewer will respond within 2 business days** with either a revised decision or an explanation of why the original finding stands.
3. **Escalation**: If you are not satisfied with the response, open a GitHub Issue in the plugin-store repository with the title `[Appeal] <plugin-name> - <brief description>`. The issue will be reviewed by a senior maintainer.

Appeals are taken seriously. Automated rules include false-positive filtering, but edge cases exist. If a static rule flagged a placeholder value (e.g., `0xYourPrivateKeyHere`) or a documentation example rather than real code, provide that context in your appeal and it will be resolved quickly.
