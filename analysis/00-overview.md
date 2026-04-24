# Codex 工程模块总览

Codex 是 OpenAI 的 CLI AI 编程助手。工程采用 **Rust 核心引擎 + Node.js CLI 封装 + TypeScript/Python SDK** 的混合架构，使用 **Bazel** 构建系统。

## 仓库顶层结构

| 目录 | 说明 |
|------|------|
| `codex-rs/` | Rust 工作空间，包含 97 个 crate，是核心引擎 |
| `codex-cli/` | npm 元包 `@openai/codex` 及 Docker 沙箱 |
| `sdk/` | TypeScript SDK + Python SDK + Python Runtime |
| `tools/` | 开发者工具（argument-comment-lint） |
| `scripts/` | 构建/CI/安装脚本 |
| `docs/` | 文档 |
| `patches/` | 第三方依赖补丁 |
| `third_party/` | 第三方代码（V8 等） |

---

## 模块分类总览

### A. 应用入口（6 个二进制 crate）

| 模块 | 路径 | 说明 |
|------|------|------|
| codex-cli | `codex-rs/cli/` | 主 CLI 二进制，用户入口 |
| codex-app-server | `codex-rs/app-server/` | 应用服务器，WebSocket/JSON-RPC 中心 |
| codex-tui | `codex-rs/tui/` | 终端 UI（TUI），富交互界面 |
| codex-mcp-server | `codex-rs/mcp-server/` | MCP 协议服务器，通过 stdio 暴露能力 |
| codex-exec | `codex-rs/exec/` | exec 子命令，连接 app-server 输出结果 |
| codex-file-search | `codex-rs/file-search/` | 模糊文件路径搜索工具 |

### B. 核心 Agent 与会话（8 个 crate）

| 模块 | 路径 | 说明 |
|------|------|------|
| codex-core | `codex-rs/core/` | **中心库** — Agent 循环、会话管理、上下文构建、插件/技能加载、MCP 工具管理、执行策略、配置加载 |
| codex-thread-store | `codex-rs/thread-store/` | 线程（会话）持久化接口，支持本地文件系统和远程 RPC |
| codex-rollout | `codex-rs/rollout/` | Rollout 文件发现与元数据管理 |
| codex-rollout-trace | `codex-rs/rollout-trace/` | Trace bundle 格式，用于重放和分析 Agent 执行 |
| codex-state | `codex-rs/state/` | SQLite 状态存储，管理 rollout 元数据 |
| codex-hooks | `codex-rs/hooks/` | 生命周期 Hook 引擎 |
| codex-features | `codex-rs/features/` | 功能标志注册中心 |
| codex-config | `codex-rs/config/` | 配置加载与解析（config.toml, permissions.toml 等） |

### C. 协议与 API 层（9 个 crate）

| 模块 | 路径 | 说明 |
|------|------|------|
| codex-protocol | `codex-rs/protocol/` | 核心协议类型：ThreadId, ToolName, AgentPath 等 |
| codex-app-server-protocol | `codex-rs/app-server-protocol/` | App Server JSON-RPC v1/v2 协议定义 |
| codex-api | `codex-rs/codex-api/` | 高层 OpenAI/Codex 后端 API 客户端 |
| codex-client | `codex-rs/codex-client/` | 低层 HTTP 传输客户端（reqwest, SSE, 重试） |
| codex-codex-mcp | `codex-rs/codex-mcp/` | MCP 协议集成层 |
| codex-rmcp-client | `codex-rs/rmcp-client/` | 基于 rmcp 的 MCP 客户端实现 |
| codex-backend-openapi-models | `codex-rs/codex-backend-openapi-models/` | 后端 API 自动生成的 OpenAPI 模型 |
| codex-backend-client | `codex-rs/backend-client/` | Codex 云服务后端 API 客户端 |
| codex-codex-experimental-api-macros | `codex-rs/codex-experimental-api-macros/` | 实验性 API 过程宏 |

### D. 认证与身份（6 个 crate）

| 模块 | 路径 | 说明 |
|------|------|------|
| codex-login | `codex-rs/login/` | OAuth 设备码/API Key 认证，Token 刷新 |
| codex-agent-identity | `codex-rs/agent-identity/` | Agent 身份密钥管理（Ed25519/Curve25519） |
| codex-device-key | `codex-rs/device-key/` | 设备绑定密钥（P-256 ECDSA） |
| codex-keyring-store | `codex-rs/keyring-store/` | 平台原生凭据存储（Keychain/Credential Manager/secret-service） |
| codex-aws-auth | `codex-rs/aws-auth/` | AWS SigV4 请求签名 |
| codex-otel | `codex-rs/otel/` | OpenTelemetry 集成（Metrics, Tracing, OTLP） |

### E. 执行与沙箱（9 个 crate）

| 模块 | 路径 | 说明 |
|------|------|------|
| codex-exec-server | `codex-rs/exec-server/` | 执行后端抽象层，管理进程和文件系统操作 |
| codex-arg0 | `codex-rs/arg0/` | 基于 PATH 的子进程调度机制 |
| codex-sandboxing | `codex-rs/sandboxing/` | 沙箱管理器（Bubblewrap, Landlock, Seatbelt） |
| codex-linux-sandbox | `codex-rs/linux-sandbox/` | Linux 沙箱（Landlock LSM + seccomp） |
| codex-windows-sandbox | `codex-rs/windows-sandbox-rs/` | Windows 沙箱（ConPTY + Job Objects） |
| codex-apply-patch | `codex-rs/apply-patch/` | 代码补丁应用，tree-sitter 解析 diff |
| codex-shell-command | `codex-rs/shell-command/` | Shell 命令解析与安全检查 |
| codex-shell-escalation | `codex-rs/shell-escalation/` | Unix 权限提升（execve-wrapper） |
| codex-process-hardening | `codex-rs/process-hardening/` | 进程加固（禁用 core dump, ptrace 等） |

### F. 执行策略（2 个 crate）

| 模块 | 路径 | 说明 |
|------|------|------|
| codex-execpolicy | `codex-rs/execpolicy/` | Starlark 执行策略引擎 |
| codex-execpolicy-legacy | `codex-rs/execpolicy-legacy/` | 旧版执行策略引擎 |

### G. 模型提供商与 AI 集成（9 个 crate）

| 模块 | 路径 | 说明 |
|------|------|------|
| codex-model-provider-info | `codex-rs/model-provider-info/` | AI 模型提供商注册表 |
| codex-model-provider | `codex-rs/model-provider/` | 模型提供商抽象（OpenAI, Bedrock 等） |
| codex-models-manager | `codex-rs/models-manager/` | 模型目录管理（预设、协作模式） |
| codex-chatgpt | `codex-rs/chatgpt/` | ChatGPT 集成 |
| codex-lmstudio | `codex-rs/lmstudio/` | LM Studio OSS 提供商集成 |
| codex-ollama | `codex-rs/ollama/` | Ollama OSS 模型提供商集成 |
| codex-realtime-webrtc | `codex-rs/realtime-webrtc/` | macOS WebRTC 实时语音交互 |
| codex-response-debug-context | `codex-rs/response-debug-context/` | API 请求调试上下文剥离 |
| codex-responses-api-proxy | `codex-rs/responses-api-proxy/` | Responses API 轻量 HTTP 代理 |

### H. 插件与技能系统（5 个 crate）

| 模块 | 路径 | 说明 |
|------|------|------|
| codex-plugin | `codex-rs/plugin/` | 共享插件标识符与元数据 |
| codex-core-plugins | `codex-rs/core-plugins/` | 插件市场管理、生命周期 |
| codex-core-skills | `codex-rs/core-skills/` | 技能引擎（加载、解析、渲染斜杠命令） |
| codex-skills | `codex-rs/skills/` | 内置技能定义 |
| codex-tools | `codex-rs/tools/` | 共享工具定义（apply-patch, exec 等） |

### I. 网络与连接（6 个 crate）

| 模块 | 路径 | 说明 |
|------|------|------|
| codex-connectors | `codex-rs/connectors/` | ChatGPT 连接器集成 |
| codex-network-proxy | `codex-rs/network-proxy/` | 网络代理服务器（HTTP CONNECT, SOCKS5, MITM） |
| codex-cloud-requirements | `codex-rs/cloud-requirements/` | 后端云需求获取（Business/Enterprise） |
| codex-cloud-tasks | `codex-rs/cloud-tasks/` | 云任务管理 TUI |
| codex-cloud-tasks-client | `codex-rs/cloud-tasks-client/` | 云任务后端 API 客户端 |
| codex-cloud-tasks-mock-client | `codex-rs/cloud-tasks-mock-client/` | 云任务 Mock 客户端 |

### J. 客户端/服务器通信（3 个 crate）

| 模块 | 路径 | 说明 |
|------|------|------|
| codex-app-server-client | `codex-rs/app-server-client/` | App Server 客户端 |
| codex-app-server-test-client | `codex-rs/app-server-test-client/` | App Server 测试客户端 |
| codex-debug-client | `codex-rs/debug-client/` | 调试客户端 CLI |

### K. 工具库（23 个 utils crate）

| 模块 | 路径 | 说明 |
|------|------|------|
| codex-utils-absolute-path | `codex-rs/utils/absolute-path/` | 保证绝对路径的类型 |
| codex-utils-approval-presets | `codex-rs/utils/approval-presets/` | 内置审批/沙箱预设 |
| codex-utils-cache | `codex-rs/utils/cache/` | LRU 缓存 |
| codex-utils-cargo-bin | `codex-rs/utils/cargo-bin/` | Bazel/cargo 环境二进制定位 |
| codex-utils-cli | `codex-rs/utils/cli/` | 共享 CLI 参数类型 |
| codex-utils-elapsed | `codex-rs/utils/elapsed/` | 人类可读时长格式化 |
| codex-utils-fuzzy-match | `codex-rs/utils/fuzzy-match/` | 模糊匹配器 |
| codex-utils-home-dir | `codex-rs/utils/home-dir/` | Codex 主目录解析 |
| codex-utils-image | `codex-rs/utils/image/` | 图像处理（缩放、编码） |
| codex-utils-json-to-toml | `codex-rs/utils/json-to-toml/` | JSON 到 TOML 转换 |
| codex-utils-oss | `codex-rs/utils/oss/` | OSS 提供商工具 |
| codex-utils-output-truncation | `codex-rs/utils/output-truncation/` | 输出截断（TruncationPolicy） |
| codex-utils-path | `codex-rs/utils/path-utils/` | 路径标准化与原子写入 |
| codex-utils-plugins | `codex-rs/utils/plugins/` | 插件路径解析 |
| codex-utils-pty | `codex-rs/utils/pty/` | PTY 进程生成 |
| codex-utils-readiness | `codex-rs/utils/readiness/` | 异步就绪标志 |
| codex-utils-rustls-provider | `codex-rs/utils/rustls-provider/` | rustls crypto 提供者 |
| codex-utils-sandbox-summary | `codex-rs/utils/sandbox-summary/` | 沙箱策略摘要 |
| codex-utils-sleep-inhibitor | `codex-rs/utils/sleep-inhibitor/` | 防休眠（跨平台） |
| codex-utils-stream-parser | `codex-rs/utils/stream-parser/` | 流式文本解析器 |
| codex-utils-string | `codex-rs/utils/string/` | 字符串工具 |
| codex-utils-template | `codex-rs/utils/template/` | {{占位符}} 模板引擎 |

### L. 支撑基础设施（14 个 crate）

| 模块 | 路径 | 说明 |
|------|------|------|
| codex-code-mode | `codex-rs/code-mode/` | V8 运行时 code mode（exec/wait） |
| codex-v8-poc | `codex-rs/v8-poc/` | V8 概念验证 |
| codex-analytics | `codex-rs/analytics/` | 分析事件收集 |
| codex-feedback | `codex-rs/feedback/` | 用户反馈提交（Sentry） |
| codex-ansi-escape | `codex-rs/ansi-escape/` | ANSI 转义码转 TUI 文本 |
| codex-async-utils | `codex-rs/async-utils/` | OrCancelExt 异步工具 |
| codex-collaboration-mode-templates | `codex-rs/collaboration-mode-templates/` | 协作模式模板 |
| codex-git-utils | `codex-rs/git-utils/` | Git 集成（分支、diff、repo 根） |
| codex-install-context | `codex-rs/install-context/` | 安装方式检测 |
| codex-secrets | `codex-rs/secrets/` | 密钥管理（age 加密，keyring） |
| codex-stdio-to-uds | `codex-rs/stdio-to-uds/` | stdio 到 Unix 域套接字中继 |
| codex-terminal-detection | `codex-rs/terminal-detection/` | 终端模拟器检测 |
| codex-test-binary-support | `codex-rs/test-binary-support/` | 测试基础设施 |
| codex-uds | `codex-rs/uds/` | 跨平台 Unix 域套接字 |

### M. CLI 封装 (`codex-cli/`)

| 组件 | 路径 | 说明 |
|------|------|------|
| npm 元包 | `codex-cli/package.json` | `@openai/codex` npm 包定义 |
| Node.js 启动器 | `codex-cli/bin/codex.js` | 平台检测、二进制解析、进程管理 |
| ripgrep 清单 | `codex-cli/bin/rg` | 按需下载 ripgrep 的 DotSlash 清单 |
| Docker 沙箱 | `codex-cli/Dockerfile` | 容器化沙箱镜像 |
| 防火墙初始化 | `codex-cli/scripts/init_firewall.sh` | iptables 出站流量限制 |
| 容器运行器 | `codex-cli/scripts/run_in_container.sh` | Docker 中运行 codex |
| npm 打包器 | `codex-cli/scripts/build_npm_package.py` | 生成 npm tgz |
| 原生依赖安装 | `codex-cli/scripts/install_native_deps.py` | 下载 Rust 二进制 |

### N. SDK（3 个包）

| 包 | 路径 | 语言 | 包名 | 说明 |
|----|------|------|------|------|
| TypeScript SDK | `sdk/typescript/` | TypeScript | `@openai/codex-sdk` | 嵌入 CLI 到 Node.js 工作流 |
| Python SDK | `sdk/python/` | Python | `codex-app-server-sdk` | App Server JSON-RPC v2 客户端 |
| Python Runtime | `sdk/python-runtime/` | Python | `openai-codex-cli-bin` | 带原生二进制分发的运行时包 |

### O. 开发者工具

| 工具 | 路径 | 说明 |
|------|------|------|
| argument-comment-lint | `tools/argument-comment-lint/` | 自研 Rust Dylint，强制 `/*参数名*/` 注释 |

### P. 脚本

| 脚本 | 路径 | 说明 |
|------|------|------|
| asciicheck.py | `scripts/` | 非 ASCII 字符检查与修复 |
| check_blob_size.py | `scripts/` | CI 文件大小策略检查 |
| check-module-bazel-lock.sh | `scripts/` | Bazel lock 文件一致性检查 |
| debug-codex.sh | `scripts/` | VS Code 调试便捷脚本 |
| list-bazel-clippy-targets.sh | `scripts/` | Clippy 目标列表 |
| list-bazel-release-targets.sh | `scripts/` | Release 目标列表 |
| mock_responses_websocket_server.py | `scripts/` | 模拟 WebSocket 响应服务器 |
| readme_toc.py | `scripts/` | README 目录自动生成 |
| stage_npm_packages.py | `scripts/` | 发布用 npm 包构建 |
| start-codex-exec.sh | `scripts/` | 远程 exec server 启动 |
| test-remote-env.sh | `scripts/` | Docker 远程测试环境 |
| install.ps1 / install.sh | `scripts/install/` | 平台安装器 |

---

## 关键架构原则

1. **Rust 为核心** — 所有核心逻辑（Agent 循环、执行、沙箱、模型交互）均在 Rust 中实现
2. **Bazel 构建** — 使用 Bazel + rules_rs 统一管理 Rust 依赖和构建
3. **平台原生** — 通过沙箱（Landlock/Seatbelt/ConPTY）、密钥链、安装器等深度适配各平台
4. **协议分层** — JSON-RPC v2 协议连接前端（TUI/CLI/SDK）与后端（app-server）
5. **多模型支持** — 支持 OpenAI、Amazon Bedrock、LM Studio、Ollama 等多种 AI 提供商

---

## 详细分析文件索引

| 文件 | 内容 |
|------|------|
| [01-core-agent.md](01-core-agent.md) | 核心 Agent 引擎、会话管理、配置、Hook、功能标志 |
| [02-protocol-api.md](02-protocol-api.md) | 协议层、API 客户端、后端通信、MCP 集成 |
| [03-auth-identity.md](03-auth-identity.md) | 认证、身份密钥、凭据存储、OTel 遥测 |
| [04-execution-sandbox.md](04-execution-sandbox.md) | 执行后端、沙箱实现、Shell 命令、补丁应用 |
| [05-execpolicy.md](05-execpolicy.md) | Starlark 与传统执行策略引擎 |
| [06-model-providers.md](06-model-providers.md) | 模型提供商、模型管理、OSS 集成 |
| [07-plugins-skills.md](07-plugins-skills.md) | 插件系统、技能引擎、工具定义 |
| [08-connectivity.md](08-connectivity.md) | 网络代理、云需求、连接器、云任务 |
| [09-client-server.md](09-client-server.md) | App Server 客户端、测试客户端、调试客户端 |
| [10-utilities.md](10-utilities.md) | 23 个工具库详细分析 |
| [11-infrastructure.md](11-infrastructure.md) | 支撑基础设施（Code Mode、分析、反馈、Git 等） |
| [12-application-binaries.md](12-application-binaries.md) | 6 个应用入口二进制 |
| [13-codex-cli-package.md](13-codex-cli-package.md) | npm CLI 封装、Docker 沙箱、安装器 |
| [14-sdk.md](14-sdk.md) | TypeScript SDK、Python SDK、Python Runtime |
| [15-tools.md](15-tools.md) | argument-comment-lint 开发者工具 |
| [16-scripts.md](16-scripts.md) | 全部构建、CI、安装脚本 |
