# OKX Plugin Store

[English](README.md) | [中文](README-ZH.md)

发现、安装和构建用于 DeFi、交易和 Web3 的 AI 代理插件。

**支持平台：** Claude Code、Cursor、OpenClaw

## 安装 Plugin Store

```bash
npx skills add okx/plugin-store --skill plugin-store
```

将 Plugin Store 技能安装到你的 AI 代理中，实现插件发现和管理功能。

## 安装插件

```bash
# 浏览所有可用插件
npx skills add okx/plugin-store

# 安装指定插件
npx skills add okx/plugin-store --skill <plugin-name>
```

---

## 按分类浏览

| 分类 | 插件 |
|------|------|
| 交易 | uniswap-ai, uniswap-swap-planner, uniswap-swap-integration |
| DeFi | uniswap-liquidity-planner, uniswap-pay-with-any-token, uniswap-cca-configurator, uniswap-cca-deployer |
| 预测 | polymarket-agent-skills |
| 开发工具 | uniswap-v4-security-foundations, uniswap-viem-integration, plugin-store |
| 自动交易 | meme-trench-scanner, top-rank-tokens-sniper, smart-money-signal-copy-trade |
| 其他 | okx-buildx-hackathon-agent-track |

## 按风险等级浏览

| 等级 | 含义 | 插件 |
|------|------|------|
| 🟢 入门 | 安全探索。仅包含只读查询、规划工具和文档，不涉及交易。 | plugin-store, okx-buildx-hackathon-agent-track, uniswap-swap-planner, uniswap-liquidity-planner, uniswap-v4-security-foundations, uniswap-viem-integration |
| 🟡 标准 | 执行交易前需用户确认。签名或发送前始终会征求同意。 | uniswap-ai, uniswap-swap-integration, uniswap-pay-with-any-token, uniswap-cca-configurator, uniswap-cca-deployer, polymarket-agent-skills |
| 🔴 高级 | 自动化交易策略。使用前需充分了解相关金融风险。 | meme-trench-scanner, top-rank-tokens-sniper, smart-money-signal-copy-trade |

## 信任标识

| 徽章 | 来源 | 含义 |
|------|------|------|
| 🟢 官方 | plugin-store | 由 OKX 开发和维护 |
| 🔵 认证合作伙伴 | uniswap-\*, polymarket-\* | 由协议团队自行发布 |
| ⚪ 社区 | 其他所有插件 | 社区贡献；使用前请自行审查 |

---

## 文档

| 你是... | English | 中文 |
|---------|---------|------|
| 插件用户 | [FOR-USERS.md](docs/FOR-USERS.md) | [FOR-USERS-ZH.md](docs/FOR-USERS-ZH.md) |
| 插件开发者 | [FOR-DEVELOPERS.md](docs/FOR-DEVELOPERS.md) | [FOR-DEVELOPERS-ZH.md](docs/FOR-DEVELOPERS-ZH.md) |
| OKX/合作伙伴团队 | [FOR-PARTNERS.md](docs/FOR-PARTNERS.md) | [FOR-PARTNERS-ZH.md](docs/FOR-PARTNERS-ZH.md) |
| 审查标准 | [REVIEW-GUIDELINES.md](docs/REVIEW-GUIDELINES.md) | [REVIEW-GUIDELINES-ZH.md](docs/REVIEW-GUIDELINES-ZH.md) |

## 贡献

提交插件请参阅 [FOR-DEVELOPERS.md](docs/FOR-DEVELOPERS.md)（[中文](docs/FOR-DEVELOPERS-ZH.md)）。流程为 Fork 仓库、开发插件，然后提交 Pull Request。

## 安全

如需报告安全问题，请发送邮件至 [security@okx.com](mailto:security@okx.com)。请勿就安全漏洞创建公开 Issue。

## 许可证

Apache-2.0
