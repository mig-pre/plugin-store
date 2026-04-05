# 审核指南

本文档描述了向 OKX Plugin Store 提交插件必须通过的审核流程。了解这些标准有助于您提交规范的插件并避免延误。

---

## 1. 审核流程概览

每个针对 `skills/` 目录的 Pull Request 最多经过五个顺序阶段：

```
提交 PR 至 skills/<plugin-name>/
    |
    v
+-------------------------------+
| Phase 1: Structure Validation |  plugin-lint.yml  约 30 秒
+-------------------------------+
    |
    v
+-------------------------------+
| Phase 2: Build Verification   |  plugin-build.yml  1-5 分钟（如适用）
+-------------------------------+
    |
    v
+-------------------------------+
| Phase 3: AI Code Review       |  plugin-ai-review.yml  2-5 分钟（仅供参考）
+-------------------------------+
    |
    v
+-------------------------------+
| Phase 4: Summary + Pre-flight |  plugin-summary.yml  1-2 分钟（需维护者审批）
+-------------------------------+
    |
    v
+-------------------------------+
| Phase 5: Human Review         |  skill-review.yml + CODEOWNERS  1-3 个工作日
+-------------------------------+
    |
    v
 通过  或  需要修改
```

Phase 1 或 Phase 2 失败将阻止后续阶段。Phase 3 **仅供参考** -- 不会阻止合并，但会为人工审核者提供详细的安全报告。Phase 4 需要通过 `summary-generation` GitHub 环境门控获得维护者批准。Phase 5 是最终的人工决策。

此外，`skill-review.yml` 工作流在每次 PR 和推送到 `main` 时独立运行，对官方插件执行静态检查和轻量级 AI 审核。

---

## 2. Phase 1：结构验证（`plugin-lint.yml`）

自动化 Lint 工具验证结构正确性、安全默认值和元数据一致性。此工作流使用 `pull_request_target` 以支持来自 Fork 的 PR 具有评论和标签的写入权限。

### Lint 前检查

在运行 Lint 规则之前，工作流强制执行三个结构性约束：

| 检查项 | 行为 |
|--------|------|
| **每个 PR 只能修改一个插件** | PR 只能修改一个 `skills/<name>/` 目录中的文件。修改多个插件将导致检查失败。 |
| **不得修改 `skills/` 之外的文件** | PR 不得修改 `skills/` 目录树之外的任何文件。 |
| **名称唯一性** | 对于新插件，名称不能与 `registry.json` 中已有的名称重复。 |

### OKX 组织成员检测

工作流会检测 PR 作者是否为 OKX 组织成员（通过检查源仓库是否在 `okx/` 下，或作者是否为公开的组织成员）。这决定了是否允许使用 `okx-` 和 `official-` 等保留前缀。

### Lint 规则

| 类别 | 检查项 | 严重程度 |
|------|--------|----------|
| **结构** | 插件根目录下存在 `plugin.yaml` | Error |
| **结构** | 存在 `SKILL.md`（针对 skill 类型插件） | Error |
| **结构** | `plugin.yaml` 包含有效的 YAML | Error |
| **版本一致性** | `plugin.yaml` 中的版本与 `SKILL.md` frontmatter 中声明的版本一致 | Error |
| **安全默认值** | 设置了 `PAUSED=True`（插件不得自动启动） | Error |
| **安全默认值** | 设置了 `PAPER_TRADE=True`（实盘交易必须用户主动开启） | Error |
| **安全默认值** | 设置了 `DRY_RUN=True`（破坏性操作必须用户主动开启） | Error |
| **Python 验证** | 所有 `.py` 文件通过语法检查（`ast.parse`） | Error |
| **URL 检查** | SKILL.md 中引用的所有 URL 返回 HTTP 2xx（404 为警告） | Warning |
| **分类** | 分类为以下之一：`trading`、`defi`、`game`、`prediction`、`data_tools`、`dev_tools`、`others` | Error |
| **许可证** | 许可证字段包含有效的 SPDX 标识符（如 `MIT`、`Apache-2.0`） | Error |
| **名称唯一性** | 新插件名称不与现有注册表条目冲突 | Error |

### 标签

工作流根据结果自动添加 PR 标签：

| 条件 | 添加标签 | 移除标签 |
|------|----------|----------|
| 新插件提交 | `new-plugin` | -- |
| 更新现有插件 | `plugin-update` | -- |
| Lint 通过 | `structure-validated` | `needs-fix` |
| Lint 失败 | `needs-fix` | `structure-validated` |

### 严重程度级别

- **Error** -- 阻止合并。必须修复后才能继续。
- **Warning** -- 仅供参考。标记以引起注意但不阻止合并。

---

## 3. Phase 2：构建验证（`plugin-build.yml`）

如果 `plugin.yaml` 包含 `build` 部分，CI 系统会克隆您的源代码并进行编译。这遵循类似 Homebrew 的模型：源代码位于您的外部仓库，plugin-store CI 负责构建和分发产物。

### 源代码模式

构建系统支持两种源代码模式：

| 模式 | 触发条件 | 行为 |
|------|----------|------|
| **外部** | `plugin.yaml` 中设置了 `build.source_repo` | 在 `build.source_commit`（固定 SHA）处克隆外部仓库 |
| **本地** | 未设置 `source_repo` | 直接使用 `skills/<name>/` 目录中的源代码 |

### 支持的语言

| 语言 | 构建工具 | 触发条件 | 安全审计 |
|------|----------|----------|----------|
| Rust | `cargo build --release` | `build.lang: rust` | `cargo audit` |
| Go | `go build -ldflags="-s -w"` | `build.lang: go` | `govulncheck` |
| TypeScript | `bun build --compile` | `build.lang: typescript` | -- |
| Node.js | `bun build --compile` | `build.lang: node` | -- |
| Python | `pip install` + 入口点验证 | `build.lang: python` | `pip-audit` |

### 构建管道步骤

对于每种支持的语言，构建任务会：

1. **准备源代码** -- 在固定 commit 处克隆外部仓库或复制本地源代码
2. **获取依赖** -- `cargo fetch`、`go mod download`、`bun install` 或 `pip install`
3. **运行安全审计** -- 语言特定的漏洞扫描器（如可用）
4. **编译** -- 生成二进制产物
5. **验证产物** -- 检查预期的二进制文件是否存在，记录大小和 SHA256
6. **上传产物** -- 将构建输出存储为 GitHub Actions artifact

### 构建报告

合并报告将作为 PR 评论发布，显示：
- 插件名称、语言和源仓库及 commit SHA
- 构建结果（通过/失败/跳过）
- 源 commit 链接，便于审计

### 跳过构建的情况

如果 `plugin.yaml` 中没有 `build` 部分（例如纯 SKILL.md 插件，无编译二进制文件），Phase 2 将被完全跳过。

---

## 4. Phase 3：AI 代码审核（`plugin-ai-review.yml`）

AI 审核者在九个维度上执行结构化审计，生成详细报告并以 PR 评论形式发布。此阶段**仅供参考** -- 不会导致 PR 检查失败。

### 环境门控

此工作流在 `ai-review` GitHub 环境下运行。使用 `ANTHROPIC_API_KEY`（或已配置的 `OPENROUTER_API_KEY`）调用 Claude API。工作流自动选择最佳可用的 Claude 模型（优先 Opus，回退至 Sonnet）。

### 实时 onchainos 上下文

审核前，工作流会克隆最新的 onchainos 源代码（`okx/onchainos-skills` main 分支）作为审核上下文。这确保 AI 准确知道哪些 CLI 命令存在，并能验证插件是否正确引用了真实的 onchainos 功能。

### 九项审计维度

| # | 维度 | 评估内容 |
|---|------|----------|
| 1 | **插件概览** | 名称、版本、分类、作者、许可证、风险级别，以及插件功能的通俗描述。 |
| 2 | **架构分析** | 组件结构（skill/binary）、SKILL.md 组织、数据流和外部依赖。 |
| 3 | **权限分析** | 推断的权限：使用的 onchainos 命令、检测到的钱包操作、调用的外部 API/URL 以及操作的链。 |
| 4 | **OnchainOS 合规性** | 所有链上写入操作（签名、广播、兑换、授权、转账）是否使用 onchainos CLI 而非自行使用原始库实现。这是**最重要的检查**。 |
| 5 | **安全评估** | 应用静态安全规则、LLM 语义判断器和有毒流检测（见下文）。包括提示注入扫描、危险操作检查和数据外泄风险分析。 |
| 6 | **源代码审查** | 语言和构建配置、依赖审计、代码安全检查（硬编码密钥、未声明的网络请求、文件系统访问、动态代码执行、unsafe 块）。仅适用于包含源代码的插件。 |
| 7 | **代码质量** | 在五个子维度上评分：完整性（25 分）、清晰度（25 分）、安全意识（25 分）、Skill 路由（15 分）和格式（10 分）。 |
| 8 | **改进建议** | 按优先级排列的可执行改进建议列表。 |
| 9 | **总结与评分** | 一句话结论、合并建议和 0-100 的总体质量评分。 |

### 三层安全扫描

第 5 维度的安全评估使用三个互补的检测层，定义在 `.github/security-rules/` 中。

#### 第 1 层：静态规则（`static-rules.md` 中的 28 条规则）

基于模式的扫描，分为四个严重级别。扫描器检查已知的危险模式，无需语义理解。

| 严重级别 | 数量 | 检测内容 |
|----------|------|----------|
| **Critical**（C01-C09） | 9 | 命令注入（管道到 shell）、提示注入关键词、base64/unicode 混淆、通过环境变量或命令替换的凭证外泄、密码保护压缩包下载、伪系统标签注入、HTML 注释中的隐藏指令、反引号注入含敏感路径。 |
| **High**（H01-H09） | 9 | 硬编码密钥（API 密钥、私钥、助记词）、要求输出凭证的指令、持久化机制（cron、launchctl、systemd）、访问敏感文件路径（~/.ssh、~/.aws、~/.kube）、直接金融/链上 API 操作、系统文件修改、.env 文件明文凭证存储、对话中索要凭证、CLI 参数中的签名交易数据。 |
| **Medium**（M01-M08） | 8 | 未固定版本的包安装、不可验证的运行时依赖、无边界标记的第三方内容获取、资源耗尽模式（fork bomb、无限循环）、通过 eval/exec 动态安装包、未固定版本的 Skill 链式调用、缺少不可信数据边界声明、外部数据字段无隔离透传。 |
| **Low**（L01-L02） | 2 | Agent 能力发现/枚举尝试、未声明的网络通信（原始 IP、DNS 查询、netcat）。 |

**判定逻辑：**
- 任意 Critical 发现 = **FAIL**（阻止合并）
- 任意 High 或 Medium 发现（无 Critical）= **WARN**（标记供人工审核）
- 仅 Low/Info 或无发现 = **PASS**

**Phase 3.5 重新裁决**：部分规则的严重程度取决于上下文。例如，C01（curl|sh）如果出现在 README.md 或安装脚本中而非 SKILL.md 中，则从 CRITICAL 降级为 MEDIUM。H05（金融 API 操作）默认为 INFO，仅在与其他发现组合时升级。

#### 第 2 层：LLM 语义判断器（`llm-judges.md` 中的 6 个判断器）

AI 驱动的语义分析，检测模式匹配无法发现的威胁：

| 判断器 | 严重级别 | 检测内容 |
|--------|----------|----------|
| **L-PINJ：提示注入** | Critical | 劫持 Agent 行为的隐藏指令，包括指令覆盖、伪系统标签、编码载荷、越狱尝试和通过未净化用户输入的 CLI 参数注入。 |
| **L-MALI：恶意意图** | Critical | 插件声明用途与实际行为之间的差异 -- 例如"钱包追踪器"暗中上传私钥。 |
| **L-MEMA：记忆投毒** | High | 尝试写入 Agent 记忆文件（MEMORY.md、SOUL.md）以植入跨会话后门。 |
| **L-IINJ：外部请求通知** | Info/Medium | 外部 API 或 CLI 调用。如果插件声明了不可信数据边界则为 Info；未声明则为 Medium。 |
| **L-AEXE：自主执行风险** | Info | 可能在无明确用户确认的情况下执行的操作（模糊授权词如"proceed"、"handle"、"automatically"且无确认门控）。 |
| **L-FINA：金融范围评估** | Info 至 Critical | 评估金融操作范围：只读查询（豁免）、有确认的写入（Info）、无确认的写入（High）、完全自主的资金转移（Critical）。 |

置信度低于 0.7 的结果自动丢弃。

#### 第 3 层：有毒流检测（`toxic-flows.md` 中的 5 条攻击链）

单独低/中严重程度的发现组合后形成完整攻击链：

| 有毒流 | 触发组合 | 严重级别 | 攻击模式 |
|--------|----------|----------|----------|
| **TF001** | 敏感路径访问 + 凭证外泄或未声明网络 | Critical | 从 ~/.ssh 或 ~/.aws 读取凭证，通过 HTTP/DNS/netcat 外泄。完整凭证窃取链路。 |
| **TF002** | 提示注入 + 持久化机制 | Critical | 绕过 Agent 安全护栏，然后注册持久服务（cron/launchctl），重启后存活。 |
| **TF004** | 不可验证依赖 + 检测到恶意意图 | High | 恶意插件安装额外未验证包，其 postinstall 钩子执行攻击载荷。 |
| **TF005** | 命令注入（curl 管道 sh）+ 金融 API 访问 | Critical | 远程脚本（可随时替换）结合金融操作，实现未授权资金转移。 |
| **TF006** | 缺少数据边界（M07/M08）+ 金融操作（H05） | High | 外部数据（token 名称、swap 路由）无隔离进入 Agent 上下文；攻击者通过链上字段注入指令操控交易参数。 |

### 看门狗

AI 审核完成后，看门狗步骤验证报告评论是否确实已发布到 PR。如果未找到报告（例如由于 API 速率限制），看门狗会发布警告评论并将工作流标记为失败，以提醒维护者。

### 质量评分解读

| 评分 | 含义 |
|------|------|
| 80-100 | 可以合并。未发现重大问题。 |
| 60-79 | 发现次要问题。有针对性修复后可能获批。 |
| 低于 60 | 存在重大问题。需要大幅修改后重新审核。 |

---

## 5. Phase 4：摘要 + Pre-flight（`plugin-summary.yml`）

此阶段生成面向用户的摘要并注入 Pre-flight 安全检查。需要通过 `summary-generation` GitHub 环境门控获得维护者批准。

### 执行内容

1. **解析 SKILL.md** -- 从本地提交或外部仓库（通过 plugin.yaml 中的 `components.skill.repo`）查找 SKILL.md。
2. **生成 SUMMARY.md** -- 由 Claude 生成的插件通俗描述，用于注册表展示。
3. **生成 SKILL_SUMMARY.md** -- Skill 功能的精简版本，供快速参考。
4. **注入 Pre-flight 检查** -- 运行 `inject-preflight.py`，为缺少安全 Pre-flight 部分的 SKILL.md 添加安全检查。
5. **推送更改** -- 将生成的文件和 Pre-flight 补丁提交回 PR 分支。

### 生成的文件

| 文件 | 用途 |
|------|------|
| `SUMMARY.md` | 面向用户的插件通俗描述，用于商店展示 |
| `SKILL_SUMMARY.md` | 精简的功能摘要 |
| `SKILL.md` 中的 Pre-flight 补丁 | 在危险操作前注入的安全检查 |

---

## 6. Phase 5：人工审核

自动化检查通过后，人工审核者审查提交内容。审核者分配由 `CODEOWNERS` 控制：

| 路径 | 审核团队 |
|------|----------|
| `skills/`（所有插件） | `@okx/plugin-store-reviewers` |
| `skills/uniswap-*/`、`skills/polymarket-*/` | `@okx/plugin-store-core` |
| `skills/plugin-store/` | `@okx/plugin-store-core` |

### 按风险级别的审核重点

| 插件风险级别 | 审核深度 | 审核者数量 |
|-------------|---------|-----------|
| 低（只读、数据展示） | 标准审核 SKILL.md 和元数据 | 1 位审核者 |
| 中（写入数据、调用外部 API） | 详细审核，包括数据流分析 | 1 位审核者 |
| 高/高级（金融操作、链上写入） | 完整安全审计所有代码和指令 | 需 2 位审核者 |

### 人工审核者关注点

- AI 审核报告的准确性（确认或推翻 AI 的发现）
- AI 可能遗漏的业务逻辑正确性
- 用户体验和文档质量
- 金融操作中的边缘情况
- 与现有 Plugin Store 标准的一致性

### SLA

人工审核在通过 Phase 3 后的 **1 至 3 个工作日**内完成。复杂或高风险插件如需额外审核者可能需要更长时间。

---

## 7. 绝对禁令（10 条红线）

以下情况将导致**立即拒绝**，无论其他因素如何。不可商量。

| # | 禁令 | 原因 |
|---|------|------|
| 1 | **硬编码私钥、助记词或 API 密钥** | 源代码中的凭证会永久暴露在版本历史中。 |
| 2 | **命令注入（`curl \| sh` 配合远程 URL）** | 远程脚本可随时替换，实现任意代码执行。 |
| 3 | **提示注入尝试** | 覆盖 Agent 安全护栏的指令会危及所有用户。 |
| 4 | **凭证外泄** | 任何将本地凭证（环境变量、文件）发送到外部服务器的机制。 |
| 5 | **混淆代码（base64 载荷、unicode 技巧）** | 审核者无法阅读的代码不可信任。 |
| 6 | **持久化机制（cron、launchctl、systemd）** | 后台服务在插件卸载后仍然存活，可作为长期后门。 |
| 7 | **访问敏感文件（~/.ssh、~/.aws、~/.kube、~/.gnupg）** | 没有插件有正当理由读取 SSH 密钥或云凭证。 |
| 8 | **绕过 OnchainOS 的直接金融操作（未声明）** | 所有链上写入操作必须通过 onchainos CLI。自行实现钱包签名、交易广播或 swap 执行是被禁止的。 |
| 9 | **供应链攻击（未固定版本的依赖 + 动态安装）** | 运行时安装未指定版本的包会打开持续的投毒窗口。 |
| 10 | **记忆投毒尝试** | 向 Agent 记忆文件（MEMORY.md、SOUL.md）写入内容以植入跨会话的持久指令。 |

---

## 8. 提交前检查清单

提交前将此检查清单复制到 PR 描述中：

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

## 9. 申诉流程

如果您认为审核决定有误：

1. **在 PR 中评论**，清楚解释您不同意该发现的原因。提供支持您观点的证据（代码引用、文档链接）。
2. **审核者将在 2 个工作日内回复**，给出修订后的决定或解释原始发现成立的原因。
3. **升级**：如果您对回复不满意，在 plugin-store 仓库中创建 GitHub Issue，标题为 `[Appeal] <plugin-name> - <简要描述>`。该 Issue 将由高级维护者审核。

申诉会被认真对待。自动化规则包含误报过滤，但边缘情况确实存在。如果静态规则标记了占位符值（如 `0xYourPrivateKeyHere`）或文档示例而非实际代码，请在申诉中提供该上下文，问题将会很快解决。
