# 合作伙伴指南

本指南面向希望在 OKX Plugin Store 中发布插件的 OKX 内部团队和外部合作伙伴。涵盖提交流程、信任徽章、可见性选项，以及最关键的——高风险插件政策。

所有插件提交至单一仓库 **`okx/plugin-store`**，放置在 `skills/` 目录下。没有单独的社区仓库。

---

## 1. 合作伙伴类型

| 类型 | 提交位置 | 信任徽章 | 示例 |
|------|----------|----------|------|
| OKX 内部（低风险） | 通过 OKX 组织账号提交至 `skills/` | Official | DEX 查询工具、价格源、投资组合查看器 |
| OKX 内部（高风险） | 通过个人 GitHub 账号提交至 `skills/` | Community | 自动交易策略、狙击机器人、跟单系统 |
| 外部合作伙伴 | 通过合作伙伴账号提交至 `skills/` | Verified Partner | Uniswap、Polymarket、第三方 DeFi 协议 |

**徽章含义：**
- **Official** -- 由 OKX 构建和维护。以 OKX 品牌显著展示。
- **Community** -- 独立贡献。不与 OKX 品牌关联。
- **Verified Partner** -- 由经过审核的外部合作伙伴构建。以合作伙伴品牌展示。

---

## 2. 高风险插件政策

> **这是本文档中最重要的部分。请仔细阅读。**

### 核心规则

**OKX 品牌不得与高风险交易策略产生关联。**

内部开发的高风险插件必须使用个人 GitHub 账号提交，并以社区贡献的形式展示——绝不能作为 OKX 官方产品。

### 高风险的判定标准

满足以下**任意一项**条件的插件即被归类为高风险：

- 无需用户逐笔确认的自动交易
- 跟单交易或信号跟踪
- 狙击或 MEV 策略
- 可能产生大额单笔交易且无明确限制
- 用户可能无法完全理解的复杂策略逻辑

### 处理规则

| 场景 | 提交方式 | 徽章 | 营销用语 |
|------|----------|------|----------|
| OKX 内部，低风险插件 | OKX 组织账号提交至 `skills/` | Official | 允许使用"OKX Official"品牌 |
| OKX 内部，高风险插件 | 个人 GitHub 账号提交至 `skills/` | Community | 仅限"社区展示"。**不得使用任何 OKX 品牌。** |
| 外部合作伙伴插件 | 合作伙伴账号提交至 `skills/` | Verified Partner | 允许联合品牌。必须包含"by [合作伙伴名称]"。 |

### 正确示例

OKX 内部团队开发了一个自动交易机器人。他们使用个人 GitHub 账号（如 `alice-dev`）提交。插件列表显示"社区展示"，不提及 OKX。用户看到 Community 徽章，理解这是独立贡献。

### 错误示例

同一团队使用 `okx` 组织账号提交该交易机器人。插件列表显示"OKX Official Auto-Trading Bot"。用户将高风险策略与 OKX 品牌关联。如果策略造成亏损，OKX 将承担声誉和潜在的法律责任。

**此政策不可商量。** 违反此规则的 PR 将被拒绝，无论代码质量如何。

---

## 3. 仓库结构

所有插件位于单一仓库：**`okx/plugin-store`**。

```
okx/plugin-store/
  skills/
    my-plugin/
      plugin.yaml          # 必需：插件元数据
      SKILL.md             # 必需：Skill 指令
      .claude-plugin/
        plugin.json        # 必需：Claude 插件清单
      scripts/             # 可选：Python 脚本
      references/          # 可选：参考文档
      LICENSE              # 建议包含
```

### 必需文件

| 文件 | 用途 |
|------|------|
| `plugin.yaml` | 插件元数据：名称、版本、分类、作者、许可证、构建配置 |
| `SKILL.md` | Skill 指令，包含 YAML frontmatter（name、version、description） |
| `.claude-plugin/plugin.json` | Claude 插件清单，包含版本和元数据 |

### 安装方式

用户通过 npx 安装插件：

```bash
npx @anthropic-ai/claude-code skills add <plugin-name>
```

---

## 4. 内部团队提交流程

适用于 OKX 员工和内部团队：

1. **创建分支**，遵循命名规范：
   ```
   partner/<team-name>/<plugin-name>
   ```
   示例：`partner/dex-team/swap-aggregator`

2. **将插件添加**到 `skills/<plugin-name>/` 目录，包含所有必需文件（`plugin.yaml`、`SKILL.md`、`.claude-plugin/plugin.json`，以及源代码如适用）。

3. **提交 PR**，使用标准 PR 模板。填写提交前检查清单（参见 [REVIEW-GUIDELINES.md](./REVIEW-GUIDELINES.md) 或 [REVIEW-GUIDELINES-ZH.md](./REVIEW-GUIDELINES-ZH.md)）。

4. **自动审核**按以下阶段运行：
   - **Phase 1**（`plugin-lint.yml`）：结构验证——检查文件存在性、YAML 有效性、安全默认值、名称唯一性
   - **Phase 2**（`plugin-build.yml`）：构建验证——如存在 `build` 部分则编译源代码
   - **Phase 3**（`plugin-ai-review.yml`）：AI 代码审核——使用 Claude API 的九维安全审计（仅供参考，不阻止合并）。在 `ai-review` 环境门控下运行。
   - **Phase 4**（`plugin-summary.yml`）：摘要生成 + Pre-flight 注入。在 `summary-generation` 环境门控下运行（需维护者批准）。
   - **Phase 5**：通过 CODEOWNERS 分配的人工审核。内部提交经过相同的自动化管道，但人工审核可能享有快速通道。

5. **合并和发布。** 审批通过后，插件出现在 Plugin Store 注册表中。

**提醒：** 如果您的插件是高风险的，请使用个人 GitHub 账号提交，而不是 OKX 组织账号。参见第 2 节。

---

## 5. 外部合作伙伴提交流程

适用于 OKX 外部的公司和项目：

1. **联系 OKX BD（商务拓展）团队**，表达发布插件的意愿。提供公司名称、插件概念和目标链。

2. **技术评估。** BD 团队将您与 Plugin Store 工程团队对接，进行技术可行性评审。

3. **签署合作协议**，涵盖插件维护责任、事件响应义务和品牌使用准则。

4. **获取 Verified Partner 访问权限。** 您将获得具有写入权限的 GitHub 账号或团队，可向 `okx/plugin-store` 仓库提交 PR。

5. **提交插件**至 `skills/<plugin-name>/`，遵循标准结构和 PR 模板。

6. **完整审核。** 您的提交将经过完整的自动化管道（Phase 1-4）加上人工审核。合作伙伴通道提供专属审核者，在流程中解答问题。

7. **合并和发布。** 审批通过后，插件以 Verified Partner 徽章和您的品牌出现在注册表中。

---

## 6. CI 管道详情

### 工作流文件

| 工作流 | 文件 | 阶段 | 环境门控 |
|--------|------|------|----------|
| 结构验证 | `.github/workflows/plugin-lint.yml` | 1 | 无（自动运行） |
| 构建验证 | `.github/workflows/plugin-build.yml` | 2 | 无（自动运行） |
| AI 代码审核 | `.github/workflows/plugin-ai-review.yml` | 3 | `ai-review` |
| 摘要 + Pre-flight | `.github/workflows/plugin-summary.yml` | 4 | `summary-generation` |
| 官方 Skill 审核 | `.github/workflows/skill-review.yml` | -- | 无（在所有 PR 和推送时运行） |

### 安全规则

AI 审核者使用的安全规则维护在 `.github/security-rules/` 中：

| 文件 | 内容 |
|------|------|
| `static-rules.md` | 28 条基于模式的规则（C01-C09、H01-H09、M01-M08、L01-L02） |
| `llm-judges.md` | 6 个 AI 语义判断器（L-PINJ、L-MALI、L-MEMA、L-IINJ、L-AEXE、L-FINA） |
| `toxic-flows.md` | 5 个攻击链检测器（TF001、TF002、TF004、TF005、TF006） |

### 关键脚本

| 脚本 | 用途 |
|------|------|
| `.github/scripts/inject-preflight.py` | 向 SKILL.md 注入 Pre-flight 安全检查 |
| `.github/scripts/gen-summary-prompt.py` | 构建摘要生成的 prompt |
| `.github/scripts/ai-review.py` | AI 审核辅助脚本 |
| `.github/prompts/ai-review-system.md` | AI 代码审核者的系统 prompt |

### 权限控制

访问权限通过 `.github/CODEOWNERS` 管理：

- **核心基础设施**（cli/、registry.json、.github/、.claude-plugin/）—— 仅 `@okx/plugin-store-core`
- **官方插件**（skills/plugin-store/）—— `@okx/plugin-store-core`
- **认证合作伙伴插件**（skills/uniswap-*/、skills/polymarket-*/）—— `@okx/plugin-store-core`
- **所有其他插件**（skills/）—— `@okx/plugin-store-reviewers`
- **文档**（docs/、README.md）—— `@okx/plugin-store-core`

---

## 7. 展示与可见性

| 展示位置 | 资格 | 获取方式 |
|----------|------|----------|
| README 精选表格 | Official 和 Verified Partner 插件 | 合并后自动包含 |
| 分类置顶 | 给定分类中的最佳插件 | 由 Plugin Store PM 选择 |
| FOR-USERS.md 案例研究 | 任何具有典型使用示例的插件 | 在 PR 中提交 3 行使用示例 |
| 网站精选 | PM 批准的、具有广泛吸引力的插件 | 直接联系 Plugin Store PM |

展示位置每季度审核一次。高质量、积极维护且用户反馈良好的插件将获得优先考虑。

---

## 8. 所需信息

### 内部团队提交表单

在 PR 描述或随附文档中提供以下信息：

| 字段 | 说明 |
|------|------|
| 插件名称 | 简短的描述性名称（小写，允许连字符） |
| 描述 | 插件功能的一段话摘要 |
| 分类 | 以下之一：`trading`、`defi`、`game`、`prediction`、`data_tools`、`dev_tools`、`others` |
| 风险级别 | `low`、`medium` 或 `high`（分类标准参见第 2 节） |
| 策略概述 | 仅供内部使用的策略逻辑描述（不会公开发布） |
| 使用示例 | 三个展示插件使用方式的命令或工作流示例 |
| 目标日期 | 计划发布日期 |
| 提交方式 | OKX 组织账号（仅限低风险）或个人账号（高风险） |
| GitHub 账号 | 拥有该提交的 GitHub 用户名 |

### 外部合作伙伴提交表单

| 字段 | 说明 |
|------|------|
| 公司名称 | 法律实体名称 |
| 联系人 | 主要技术联系人的姓名和邮箱 |
| 插件描述 | 插件功能及其对用户价值的详细描述 |
| 支持的链 | 插件交互的区块链列表 |
| API 文档链接 | 您的 API 文档 URL（如果插件调用您的 API） |
| 品牌素材 | Logo（推荐 SVG 格式）和用于商城展示的标语 |
| 目标发布日期 | 计划发布日期 |

---

## 9. 事件响应

如果已发布的插件被发现存在安全漏洞、恶意行为或严重 bug，将按以下时间线处理：

| 时间范围 | 操作 |
|----------|------|
| **立即** | 在 `registry.json` 中将插件标记为 `suspended`。警告用户不要安装或使用。 |
| **1 小时内** | 合并 PR 以禁用安装。现有安装在 CLI 中标记安全警告。 |
| **24 小时内** | 完成根因分析。通知插件作者（内部团队或外部合作伙伴）并提供调查结果。 |
| **后续跟进** | 作者提交修复，插件重新经过完整审核管道。如果问题无法解决，插件将从注册表中永久移除。 |

### 合作伙伴在事件中的责任

- **内部团队**：在工作时间内 1 小时内响应事件通道。24 小时内提供修复或缓解方案。
- **外部合作伙伴**：在工作时间内 4 小时内响应（依据合作协议）。48 小时内提供修复，否则插件将被永久移除。

重复事件（6 个月内 3 次或以上）可能导致撤销发布权限。
