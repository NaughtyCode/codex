# 支撑基础设施

涵盖 14 个 crate：`codex-code-mode`, `codex-v8-poc`, `codex-analytics`, `codex-feedback`, `codex-ansi-escape`, `codex-async-utils`, `codex-collaboration-mode-templates`, `codex-git-utils`, `codex-install-context`, `codex-secrets`, `codex-stdio-to-uds`, `codex-terminal-detection`, `codex-test-binary-support`, `codex-uds`

---

## codex-code-mode — Code Mode 执行

**路径**: `codex-rs/code-mode/`

管理 V8 JavaScript 运行时代码模式：

- **exec 工具**: 在 V8 沙箱中运行 JavaScript 代码
- **wait 工具**: 在 JS 环境中等待异步操作
- **工具定义**: 代码模式特定工具注册
- **响应处理**: 处理 JS 执行结果并格式化输出

---

## codex-v8-poc — V8 概念验证

**路径**: `codex-rs/v8-poc/`

Bazel-wired 概念验证，用于未来的 V8 实验：
- 最小化 crate，仅返回 V8 版本号
- 为未来的 V8 深度集成做准备

---

## codex-analytics — 事件收集

**路径**: `codex-rs/analytics/`

分析事件收集和传输：

| 事件类型 | 说明 |
|----------|------|
| Guardian 审查 | 自动审查决策事件 |
| 应用/技能调用 | 功能和技能使用统计 |
| 压缩事件 | 上下文压缩触发和结果 |
| Turn 转向 | 用户转向操作统计 |

---

## codex-feedback — 用户反馈

**路径**: `codex-rs/feedback/`

用户反馈提交系统：

- **诊断收集**: 收集系统信息、配置和错误日志
- **Sentry 上传**: 将崩溃报告上传到 Sentry
- **反馈标签**: 用户定义的反馈标记
- **附件**: 支持附加文件和截图

---

## codex-ansi-escape — ANSI 转换

**路径**: `codex-rs/ansi-escape/`

ANSI 转义码到 TUI 文本的转换：
- 使用 `ansi-to-tui` 库进行转换
- Tab 展开用于转录渲染
- 处理终端颜色代码

---

## codex-async-utils — 异步工具

**路径**: `codex-rs/async-utils/`

`OrCancelExt` trait — 提供取消安全的 future 执行：
- `or_cancel()` — 在 CancellationToken 触发时取消 future
- 与 tokio 取消机制集成

---

## codex-collaboration-mode-templates — 协作模式模板

**路径**: `codex-rs/collaboration-mode-templates/`

内置 Markdown 模板用于协作模式：

| 模板 | 文件 |
|------|------|
| Plan | `plan.md` |
| Default | `default.md` |
| Execute | `execute.md` |
| Pair Programming | `pair_programming.md` |

---

## codex-git-utils — Git 集成

**路径**: `codex-rs/git-utils/`

Git 仓库集成：

- **分支检测**: 当前分支名检测
- **Diff 计算**: 工作区/暂存区差异计算
- **仓库根解析**: 向上遍历查找 `.git` 目录
- **提交归属**: Git 提交信息中的作者归属

---

## codex-install-context — 安装上下文

**路径**: `codex-rs/install-context/`

安装方法检测：
- 独立安装（standalone, via install.ps1/install.sh）
- npm 管理（`@openai/codex` 包）
- bun 管理
- 版本/发布目录管理

---

## codex-secrets — 密钥管理

**路径**: `codex-rs/secrets/`

安全密钥管理：

- **age 加密**: 本地密钥的 age 加密存储
- **keyring 存储**: 平台 keyring 支持的密钥存储
- **密钥清理**: 在日志和错误消息中清理密钥
- **命名约定**: 标准化的密钥命名

---

## codex-stdio-to-uds — stdio 到 UDS 中继

**路径**: `codex-rs/stdio-to-uds/`

将 stdin/stdout 中继到 Unix 域套接字：
- 用于基于 stdio 的子进程 socket 通信
- 连接生命周期管理

---

## codex-terminal-detection — 终端检测

**路径**: `codex-rs/terminal-detection/`

终端模拟器识别：

| 终端 | 检测方法 |
|------|----------|
| Apple Terminal | `TERM_PROGRAM` |
| iTerm2 | `ITERM_SESSION_ID` |
| Ghostty | `TERM` / env |
| VS Code Terminal | `TERM_PROGRAM` |
| Warp | `WARP_*` env |
| WezTerm | `TERM_PROGRAM` |
| Windows Terminal | `WT_SESSION` |

用于遥测和配置。

---

## codex-test-binary-support — 测试基础设施

**路径**: `codex-rs/test-binary-support/`

集成测试基础设施：
- 临时目录设置
- arg0 调度配置
- XDG 环境变量设置
- 沙箱测试环境准备

---

## codex-uds — Unix 域套接字

**路径**: `codex-rs/uds/`

跨平台 Unix 域套接字抽象：
- 基于 tokio 的异步 UDS 实现
- Windows 支持 via `uds_windows`
- Unix 平台原生 UDS
