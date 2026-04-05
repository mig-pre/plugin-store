# OKX Plugin Store -- 用户指南

欢迎！本指南面向所有希望在 AI 助手中使用插件的用户。
无需编程经验。只要你能给 AI 聊天机器人发消息，就能使用插件。

---

## 目录

1. [什么是 Plugin Store？](#1-什么是-plugin-store)
2. [为什么选择 OKX Plugin Store？](#2-为什么选择-okx-plugin-store)
3. [快速上手](#3-快速上手)
4. [插件目录](#4-插件目录)
5. [使用示例](#5-使用示例)
6. [管理你的插件](#6-管理你的插件)
7. [安全与免责声明](#7-安全与免责声明)
8. [常见问题](#8-常见问题)

---

## 1. 什么是 Plugin Store？

可以把 Plugin Store 看作你 AI 助手的**应用商店**。

你的 AI 助手（如 Claude Code、Cursor 或 OpenClaw）已经很聪明了。但就像手机
安装新应用后会获得新功能一样，你的 AI 助手安装插件后也会获得新能力。

- **Plugin Store** = 应用商店
- **一个插件** = 一个应用
- **安装插件** = 在手机上下载应用

没有插件时，你的 AI 助手可以回答问题、处理一般性任务。有了插件，它可以在
去中心化交易所兑换代币、追踪资深交易员的操作、帮你规划流动性头寸等等。

### 插件的结构

商店中的每个插件都包含三个核心文件：

- **plugin.yaml** -- 插件的元数据（名称、版本、作者、描述）
- **SKILL.md** -- 教 AI 助手如何使用该插件的指令文件
- **.claude-plugin/plugin.json** -- 插件系统的配置文件

你无需关心这些文件的细节。这里提及只是为了让你了解安装插件时背后发生了什么。

### 预检自动注入

通过 Plugin Store 安装插件时，系统会在后台自动配置你所需的一切：

- **onchainos CLI** -- 链上操作系统的命令行工具
- **onchainos Skills** -- 区块链交互的核心 AI 技能
- **plugin-store Skill** -- 插件管理技能本身
- **安装报告** -- 已安装和已配置内容的摘要

这些无需手动安装，在每次会话中首次安装插件时会自动注入。

### 实际效果

下面展示安装和使用插件的过程。你只需用自然语言描述需求，AI 助手会完成剩余工作：

```
你:     我想在 Uniswap 上规划一笔代币兑换。

AI:     我可以帮你！让我先安装 Uniswap Swap Planner 插件。

        正在安装 uniswap-swap-planner... 完成！

        你想兑换哪些代币？例如，你可以在以太坊主网上将 ETH 兑换为 USDC。

你:     我想把 0.5 ETH 兑换成 USDC。

AI:     这是你的兑换方案：

        - 来源: 0.5 ETH
        - 目标: ~975 USDC（预估）
        - 网络: Ethereum
        - 预估手续费: ~$2.40

        这是在 Uniswap 上执行此兑换的链接：
        [在 Uniswap App 中打开]

        在执行前你需要在自己的钱包中确认交易。
```

就这么简单。你描述了需求，AI 完成了剩下的一切。

---

## 2. 为什么选择 OKX Plugin Store？

### 每个插件都经过 4 阶段安全审核

在任何插件上架之前，都必须通过四层检查：

1. **自动化代码扫描** -- 程序会扫描插件中是否存在已知的危险模式（如试图窃取
   密码或注入隐藏命令的代码）。可以把它想象成机场安检扫描仪。

2. **AI 驱动的行为分析** -- AI 审核员会阅读整个插件，确保它确实在做它声称要做的
   事，而不是别的。这能发现简单扫描器可能遗漏的问题。

3. **毒性流检测** -- 系统会检查多个小动作组合起来是否会形成有害行为。例如，
   一个既读取敏感文件又向互联网发送数据的插件会被标记，即使每个单独的动作
   看起来都无害。

4. **人工审核** -- 真人在插件上线前进行审核。自动化工具很好，但人的判断力能
   发现机器无法捕捉的问题。

### 三个信任等级告诉你插件的来源

不是所有插件都来自同一个地方。我们为每个插件标注来源，让你了解其出处：

| 徽章 | 含义 | 示例 |
|------|------|------|
| **Official（官方）** | 由 OKX 团队开发和维护 | Plugin Store 本身 |
| **Verified Partner（认证合作伙伴）** | 由实际协议团队（Uniswap、Polymarket 等）开发 | Uniswap AI, Polymarket Agent |
| **Community（社区）** | 由社区独立开发者开发 | Meme Trench Scanner |

三个等级都经过相同的 4 阶段审核。徽章表示的是*谁*制作了它，而不是它是否安全。
不过，Official 和 Verified Partner 级别的插件来自有良好声誉的团队。

### 三个风险等级告诉你插件能做什么

每个插件都被分配了一个风险等级，让你知道会发生什么：

| 风险等级 | 含义 |
|---------|------|
| **Starter（入门级）** | 只读。规划、浏览和展示信息。绝不触碰你的钱包。 |
| **Standard（标准级）** | 可以准备交易，但始终会先请求你的确认。 |
| **Advanced（高级）** | 自动化交易。可以代你执行交易。使用前请充分了解风险。 |

### 一行命令即可安装

安装插件只需一条命令。无需下载文件，无需编辑配置文件，无需重启任何东西。
一行命令，即可开始使用。

---

## 3. 快速上手

上手大约需要 60 秒。以下是具体步骤：

### 第 1 步：安装 Plugin Store

首先，安装 Plugin Store 技能本身。这让你的 AI 助手具备浏览和管理所有可用
插件的能力。

打开你的 AI 助手的终端或聊天界面，运行：

```
npx skills add okx/plugin-store --skill plugin-store
```

### 第 2 步：打开你的 AI 助手

打开以下任一支持的 AI 助手：

- **Claude Code** -- Anthropic 的命令行 AI 工具
- **Cursor** -- AI 驱动的代码编辑器
- **OpenClaw** -- 开源 AI 助手

如果你已经打开了其中一个，可以直接进入下一步。

### 第 3 步：安装一个插件

在你的 AI 助手的终端或聊天界面中输入以下命令：

```
npx skills add okx/plugin-store --skill uniswap-swap-planner
```

将 `uniswap-swap-planner` 替换为你想要的任何插件名称。（参见下方的
[插件目录](#4-插件目录)查看全部 16 个插件的完整列表。）

所有插件安装都使用相同的模式：

```
npx skills add okx/plugin-store --skill <plugin-name>
```

### 第 4 步：开始对话

用你自己的话描述你想做什么：

```
你:     帮我在 Arbitrum 上规划一笔 100 USDC 兑换 ETH 的交易。
```

你不需要使用任何特殊命令或语法。像跟一个乐于助人的朋友说话一样跟你的 AI
助手交流就行。

### 第 5 步：AI 自动使用插件

你的 AI 助手会识别你的需求，激活合适的插件，并引导你完成操作。如果涉及交易，
你始终有机会在任何操作发生前进行审查和确认。

```
AI:     我已经为你准备了兑换方案：

        - 来源: 100 USDC
        - 目标: ~0.051 ETH（预估）
        - 网络: Arbitrum
        - 预估手续费: ~$0.08

        这是在 Uniswap 上完成此兑换的链接。你需要在钱包中批准此交易。
```

---

## 4. 插件目录

Plugin Store 目前包含 **16 个插件**。以下是帮你找到合适插件的方法。

### 按需求查找

| 我想... | 插件 | 安装命令 | 风险等级 |
|---------|------|---------|---------|
| 浏览和安装其他插件 | Plugin Store | `npx skills add okx/plugin-store --skill plugin-store` | Starter |
| 参加 AI 黑客松 | OKX BuildX Hackathon Guide | `npx skills add okx/plugin-store --skill okx-buildx-hackathon-agent-track` | Starter |
| 在 Uniswap 上规划代币兑换 | Uniswap Swap Planner | `npx skills add okx/plugin-store --skill uniswap-swap-planner` | Starter |
| 规划流动性头寸 | Uniswap Liquidity Planner | `npx skills add okx/plugin-store --skill uniswap-liquidity-planner` | Starter |
| 学习 Uniswap v4 Hook 安全知识 | Uniswap V4 Security Foundations | `npx skills add okx/plugin-store --skill uniswap-v4-security-foundations` | Starter |
| 使用区块链开发库构建应用 | Uniswap Viem Integration | `npx skills add okx/plugin-store --skill uniswap-viem-integration` | Starter |
| 用 AI 工具兑换代币 | Uniswap AI | `npx skills add okx/plugin-store --skill uniswap-ai` | Standard |
| 在项目中集成兑换功能 | Uniswap Swap Integration | `npx skills add okx/plugin-store --skill uniswap-swap-integration` | Standard |
| 用任意代币支付 | Uniswap Pay With Any Token | `npx skills add okx/plugin-store --skill uniswap-pay-with-any-token` | Standard |
| 配置代币拍卖合约 | Uniswap CCA Configurator | `npx skills add okx/plugin-store --skill uniswap-cca-configurator` | Standard |
| 部署拍卖智能合约 | Uniswap CCA Deployer | `npx skills add okx/plugin-store --skill uniswap-cca-deployer` | Standard |
| 在预测市场交易 | Polymarket Agent Skills | `npx skills add okx/plugin-store --skill polymarket-agent-skills` | Standard |
| 自动交易新上线的 meme 代币 | Meme Trench Scanner | `npx skills add okx/plugin-store --skill meme-trench-scanner` | Advanced |
| 狙击 OKX 排行榜代币 | Top Rank Tokens Sniper | `npx skills add okx/plugin-store --skill top-rank-tokens-sniper` | Advanced |
| 跟单聪明钱钱包 | Smart Money Signal Copy Trade | `npx skills add okx/plugin-store --skill smart-money-signal-copy-trade` | Advanced |
| 运行 Rust CLI 端到端测试 | Test Rust CLI | `npx skills add okx/plugin-store --skill test-rust-cli` | Starter |

### 按风险等级查找

每个插件都有一个风险等级，告诉你它能做什么以及你需要注意什么：

#### Starter（入门级）-- 安全探索

这些插件只**读取**信息。它们帮助你规划、学习和浏览，绝不触碰你的钱包或发起交易。

*示例：浏览可用插件、规划兑换方案（不执行）、阅读安全文档。*

**预期体验：** 你提出问题，AI 给你信息。没有资产变动，没有签名，没有费用。

**该等级的插件：**
- Plugin Store
- OKX BuildX Hackathon Guide
- Uniswap Swap Planner
- Uniswap Liquidity Planner
- Uniswap V4 Security Foundations
- Uniswap Viem Integration
- Test Rust CLI

---

#### Standard（标准级）-- 需要你确认的交易

这些插件可以**准备交易**，但在任何操作发生前始终会请求你的确认。可以把它想象成
购物车：插件把商品加入购物车，但由你决定是否结账。

*示例：通过 Uniswap 兑换代币、在 Polymarket 上下注、用加密货币支付服务。*

**预期体验：** AI 会向你详细说明即将发生的事情，展示所有细节（金额、手续费、
目标地址），并请求你确认。你随时可以拒绝。

**该等级的插件：**
- Uniswap AI
- Uniswap Swap Integration
- Uniswap Pay With Any Token
- Uniswap CCA Configurator
- Uniswap CCA Deployer
- Polymarket Agent Skills

---

#### Advanced（高级）-- 自动化交易（请先了解风险）

这些插件运行**自动化交易策略**。它们可以根据信号、排名或市场模式代你买卖代币。
这就像把你的交易设为自动驾驶。

*示例：自动买入聪明钱钱包正在买入的代币、狙击排行榜上新上线的代币、扫描
meme 代币机会。*

**预期体验：** 这些插件可以用真金白银执行真实交易。它们提供模拟交易模式
（不使用真钱的练习模式），让你在冒险之前了解它们的运作方式。

**重要提示：** 这些是高风险工具。代币价格可以在数秒内跌至零。你可能损失全部
投资。请务必先从模拟模式开始。

**该等级的插件：**
- Meme Trench Scanner
- Top Rank Tokens Sniper
- Smart Money Signal Copy Trade

---

## 5. 使用示例

以下是三个由简入深的真实场景，展示插件在实际中如何工作。

### 场景 1：浏览 DeFi 机会（Starter -- 安全）

你刚听说 DeFi，想了解有什么可用的。不涉及资金，只是探索。

```
你:     有哪些 DeFi 插件？我是新手，只想看看。

AI:     欢迎！以下是 Plugin Store 中目前可用的内容：

        交易
        - Uniswap AI -- 在 AI 引导下兑换代币
        - Uniswap Swap Planner -- 规划兑换并获取直接链接
        - Uniswap Swap Integration -- 在项目中集成兑换功能

        DEFI
        - Uniswap Liquidity Planner -- 规划流动性头寸
        - Uniswap Pay With Any Token -- 用任意代币支付

        预测市场
        - Polymarket Agent Skills -- 在预测市场交易

        自动化交易
        - Meme Trench Scanner -- 自动交易 meme 代币
        - Top Rank Tokens Sniper -- 狙击热门代币
        - Smart Money Signal Copy Trade -- 跟单聪明钱

        Starter 级别的插件（Swap Planner、Liquidity Planner）是很好的起点。
        它们只展示信息——不会买卖任何东西。

        要我为你安装一个试试吗？

你:     好的，试试 Swap Planner。

AI:     正在安装 uniswap-swap-planner... 完成！

        你现在可以问我这样的问题：
        - "把 1 ETH 兑换成 USDC 需要多少钱？"
        - "500 USDC 兑换 WBTC 的最佳路径是什么？"
        - "在 Arbitrum 上规划一笔兑换"

        你想试试什么？

你:     在以太坊上把 1 ETH 兑换成 USDC 需要多少钱？

AI:     这是你的兑换方案：

        - 来源: 1 ETH
        - 目标: ~1,950 USDC（预估）
        - 网络: 以太坊主网
        - 预估 Gas 费: ~$3.20
        - 价格影响: < 0.01%

        这只是一个方案——没有花费任何资金。如果你想执行这笔兑换，
        我可以生成一个链接带你到 Uniswap 应用，在那里你可以用自己的
        钱包确认交易。
```

**发生了什么：** 你浏览了插件、安装了一个，并获取了兑换信息。全程没有任何
资金风险。这就是 Starter 级别插件的体验。

---

### 场景 2：在 Uniswap 上兑换代币（Standard -- 需确认）

你有一些 ETH，想兑换成 USDC。这涉及真实交易，所以 AI 会请求你确认。

```
你:     我想在以太坊上把 0.5 ETH 兑换成 USDC。

AI:     我来帮你通过 Uniswap 准备这笔兑换。

        以下是详细信息：

        兑换摘要
        - 卖出: 0.5 ETH
        - 买入: ~975.00 USDC（预估）
        - 汇率: 1 ETH = ~1,950 USDC
        - 网络: 以太坊主网
        - 预估 Gas 费: ~$2.80
        - 最低接收量（0.5% 滑点）: ~970.12 USDC

        准备好了吗？

        我已生成 Uniswap 应用的链接，你可以在那里审查并确认这笔兑换：
        [在 Uniswap App 中打开]

        重要提示：你需要在钱包（如 MetaMask）中确认交易。
        兑换只会在你批准后才会执行。确认前你可以随时取消。

你:     看起来不错，我去点击链接。

AI:     太好了！打开链接后，流程如下：

        1. Uniswap 应用会显示兑换详情
        2. 在 Uniswap 界面中点击"Swap"
        3. 你的钱包会弹出确认请求
        4. 查看 Gas 费和总成本
        5. 在钱包中点击"确认"

        一旦在钱包中确认，兑换即为最终操作。如果你有任何问题或需要
        其他帮助，请告诉我！
```

**发生了什么：** AI 准备了交易详情，提前向你展示了所有内容，并给了你一个链接。
你仍然需要在自己的钱包中确认。AI 从未接触过你的资金——它只是帮助你规划和准备。

---

### 场景 3：追踪和跟单聪明钱（Advanced -- 附风险警告）

你想自动跟随资深交易员（称为"聪明钱"）的操作。这是一种涉及真实交易的高级策略。

```
你:     我想试试跟单交易。能自动跟随聪明钱钱包的操作吗？

AI:     我可以使用 Smart Money Signal Copy Trade 插件帮你设置。

        在开始之前，有几件重要的事需要了解：

        风险警告
        - 此插件会用真金白银执行真实交易
        - 加密货币价格可以在数秒内跌至零
        - "聪明钱"信号不是保证——即使资深交易员也会亏损
        - 你可能损失全部投资金额
        - 过去的表现不代表未来的结果

        我强烈建议先从模拟模式开始。模拟模式在不使用真钱的情况下
        模拟交易，让你在冒险之前了解策略的表现。

        你想：
        (a) 从模拟模式开始（推荐——不使用真钱）
        (b) 先了解更多工作原理
        (c) 设置实盘交易（真金白银，有风险）

你:     先用模拟模式吧。

AI:     明智的选择！让我先安装插件。

        正在安装 smart-money-signal-copy-trade... 完成！

        模拟模式设置

        机器人现已配置为模拟模式（DRY_RUN = True）。它会做以下事情：

        - 监控聪明钱钱包的买卖信号
        - 根据这些信号模拟交易
        - 追踪模拟盈亏
        - 在 http://localhost:3248 显示仪表板

        不涉及真钱。机器人会记录它本来会做什么，以便你评估策略。

        机器人还包含安全功能：
        - 开发者/打包者检测（避免可疑创建者的代币）
        - 成本感知止盈水平
        - 会话风险控制（限制总敞口）

        要我现在启动模拟交易机器人吗？

你:     好的，启动吧。

AI:     正在以模拟模式启动 Smart Money Signal Copy Trade...

        机器人正在运行。你可以：
        - 在 http://localhost:3248 查看仪表板
        - 查看持仓：仪表板显示所有追踪的信号
        - 查看交易历史：每笔模拟交易都有记录

        有信号进来时我会通知你。同时，机器人正在监控聪明钱钱包的动态。

        记住：当你准备考虑实盘交易时，我们可以先一起回顾你的模拟交易
        结果。不急。
```

**发生了什么：** AI 在做任何操作之前就警告了你风险。它推荐了模拟模式（练习模式）。
它解释了机器人的功能和监控方式。在每一步，你都掌握着决定权。

---

## 6. 管理你的插件

所有插件管理都通过 `npx skills add` 命令或用自然语言询问 AI 助手来完成。

### 安装插件

```
npx skills add okx/plugin-store --skill <plugin-name>
```

或者直接说：*"帮我安装 Uniswap Swap Planner 插件。"*

### 安装 Plugin Store 本身

```
npx skills add okx/plugin-store --skill plugin-store
```

或者说：*"安装 Plugin Store 技能。"*

### 查看已安装的插件

询问你的 AI 助手：*"我安装了哪些插件？"*

### 更新插件到最新版本

重新安装插件即可获取最新版本：

```
npx skills add okx/plugin-store --skill <plugin-name>
```

或者说：*"重新安装 Uniswap AI 插件以获取最新版本。"*

### 获取特定插件的详情

询问你的 AI 助手：*"给我介绍一下 Uniswap AI 插件。"*

### 按关键词搜索插件

询问你的 AI 助手：*"有没有用于兑换代币的插件？"*

### 列出所有可用插件

询问你的 AI 助手：*"显示所有可用的插件。"*

---

## 7. 安全与免责声明

### 我们如何保护你

OKX Plugin Store 中的每个插件在发布前都会经过 **4 阶段安全审核**：

| 阶段 | 做了什么 | 能发现什么 |
|------|---------|-----------|
| 1. 自动化代码扫描 | 程序检查危险的代码模式 | 恶意软件、密码窃取、隐藏命令 |
| 2. AI 行为分析 | AI 阅读整个插件以验证其功能符合声明 | 欺骗性插件、隐藏功能 |
| 3. 毒性流检测 | 系统检查看似无害的动作组合是否会产生有害结果 | 隐藏在表面之下的复杂攻击 |
| 4. 人工审核 | 真人审查提交的内容 | 自动化工具遗漏的任何问题 |

所有插件——Official、Verified Partner 和 Community——都经过相同的流程。

### 你需要了解的

虽然我们努力确保 Plugin Store 的安全，但有一些重要的事情需要理解：

- **这不是投资建议。** Plugin Store 及其插件是工具。它们不会告诉你该买什么、
  卖什么或投资什么。任何交易决定都由你自己做出。

- **自己做好研究（DYOR）。** 在使用任何涉及真钱的插件之前，花时间了解它的
  功能、风险以及是否适合你的情况。

- **加密货币波动剧烈。** 代币价格可以在很短的时间内大幅上涨或下跌。你可能
  损失部分或全部资金。

- **从模拟模式开始。** Advanced 级别的交易插件提供使用模拟资金的练习模式。
  务必先试用这个。

- **链上交易不可逆。** 一旦区块链交易被确认，就无法撤销。确认前请反复检查。

- **过去的表现不是保证。** 一个策略以前有效，不意味着以后也会有效。

### 如何报告问题

如果你发现某个插件行为异常、疑似恶意或存在安全问题：

- **邮箱：** [security@okx.com](mailto:security@okx.com)
- **GitHub：** 在 [github.com/okx/plugin-store](https://github.com/okx/plugin-store) 提交 issue

对于安全漏洞，请使用邮箱而非公开的 GitHub issue。这样团队有时间在问题被广泛
知晓前进行修复。

---

## 8. 常见问题

### 免费吗？

**是的。** Plugin Store 和其中所有插件均可免费安装和使用。部分插件与区块链网络
交互时你可能需要支付网络手续费（称为"Gas 费"），但插件本身不收取任何费用。

### 安全吗？

每个插件在上架前都经过 **4 阶段安全审核**（自动化扫描、AI 分析、毒性流检测和
人工审核）。我们还为每个插件标注信任徽章（Official、Verified Partner 或
Community），让你了解其来源。

也就是说，没有任何审核流程是完美的。对于涉及真钱的插件，请务必从模拟模式开始，
并自己做好研究。

### Plugin Store 收集什么数据？

**不收集任何数据。** Plugin Store 本身不收集任何个人数据、使用数据或钱包信息。
它只是一个将插件文件下载到你电脑上的工具——仅此而已。

个别插件可能会与区块链网络交互（区块链本身是公开的），但 Plugin Store 本身
不会追踪你。

### 插件安装在哪里？

插件安装在你电脑上 AI 助手的本地配置文件夹中。它们只是文本文件（给 AI 的指令），
存储在你的 AI 助手的其他设置旁边。

- 它们留在你的设备上
- 不会被上传到任何地方
- 你可以随时删除它们

### 需要会编程吗？

**不需要。** 你通过自然语言与 AI 助手交流来使用插件。AI 会处理所有技术细节。
你只需描述你想做什么。

### 支持哪些 AI 助手？

目前，Plugin Store 支持：

- **Claude Code** -- Anthropic 的命令行 AI
- **Cursor** -- AI 驱动的代码编辑器
- **OpenClaw** -- 开源 AI 助手

### 可以同时使用多个插件吗？

**可以。** 你可以安装任意数量的插件。你的 AI 助手会根据你的需求自动使用合适的
插件。例如，如果你问关于兑换代币的问题，它会使用兑换插件。如果你问关于预测市场
的问题，它会使用 Polymarket 插件。

### 插件不工作了怎么办？

尝试以下步骤：

1. **重新安装插件：** 运行
   `npx skills add okx/plugin-store --skill <plugin-name>` 获取最新版本
2. **查看已知问题：** 访问插件的 GitHub 页面查看已知问题
3. **报告问题：** 如果都不行，报告该问题（参见上方的
   [如何报告问题](#如何报告问题)）

### 什么是"模拟模式"？

模拟模式（也称为"dry run"或"模拟交易模式"）让你在不冒真钱风险的情况下测试
交易策略。插件会模拟可能发生的情况——追踪买入、卖出、盈利和亏损——但不会发生
实际交易。这是在投入真实资金前评估 Advanced 级别插件的最安全方式。

### 什么是"Gas 费"？

Gas 费是你向区块链网络支付的小额费用，用于处理你的交易。可以把它想象成寄信的
邮费——你支付少量费用来让你的交易被传送和记录。Gas 费会根据网络活跃度而变化。
只读取信息的插件（Starter 级别）不会产生 Gas 费。

### 有多少个插件可用？

目前 Plugin Store 中有 **16 个插件**，涵盖 DeFi 交易、流动性管理、预测市场
和自动化交易策略等多个类别。

---

*本指南由 OKX Plugin Store 团队维护。
开发者文档请参阅 [FOR-DEVELOPERS.md](FOR-DEVELOPERS.md)。
合作伙伴文档请参阅 [FOR-PARTNERS.md](FOR-PARTNERS.md)。*
