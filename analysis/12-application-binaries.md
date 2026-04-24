# 应用入口二进制

涵盖 6 个二进制 crate：`codex-cli`, `codex-app-server`, `codex-tui`, `codex-mcp-server`, `codex-exec`, `codex-file-search`

---

## codex-cli — 主 CLI 二进制

**路径**: `codex-rs/cli/` | **二进制名**: `codex`

用户主入口，通过 clap 实现完整的命令行解析。支持 20+ 子命令。

### 多名称二进制调度

通过 `codex-arg0` 的 `arg0_dispatch_or_else()` 实现：单个编译二进制根据 `argv[0]` 表现为不同可执行文件（`codex`、`codex-exec`、`codex-mcp-server`、`codex-linux-sandbox`）。

### 子命令系统

| 子命令 | 说明 |
|--------|------|
| _(无)_ | 交互式 TUI（默认模式） |
| `exec` | 非交互执行（`-e` 别名） |
| `review` | 代码审查 |
| `login` | 认证（API key / OAuth） |
| `logout` | 移除凭据 |
| `mcp` | MCP 服务器配置管理 |
| `mcp-server` | 启动 MCP 服务器（stdio） |
| `app-server` | 运行应用服务器 |
| `app` | 启动桌面应用（macOS/Windows） |
| `completion` | Shell 补全脚本生成 |
| `sandbox` | 沙箱命令运行 |
| `debug` | 调试工具 |
| `apply` | 应用最近 diff（`-a`） |
| `resume` | 恢复之前会话 |
| `fork` | 分叉之前会话 |
| `cloud` | 浏览云任务 |
| `exec-server` | 运行独立 exec-server |
| `features` | 功能标志管理 |

### 关键特性

- **配置传播**: 根级别 `-c` 覆盖优先于子命令标志
- **功能标志**: 通过 `--enable`/`--disable` 传递
- **远程模式**: `--remote ws://host:port` 连接到远程 app-server
- **退出处理**: 打印 token 用量和恢复命令提示
- **自更新**: 通过平台安装器脚本

---

## codex-app-server — 应用服务器

**路径**: `codex-rs/app-server/` | **二进制名**: `codex-app-server`

WebSocket/JSON-RPC 中心，管理核心 Agent 与客户端（TUI/CLI/SDK）之间的连接。

### 传输模式

| 传输 | 说明 |
|------|------|
| `Stdio` | 单客户端 stdin/stdout JSON-RPC（默认） |
| `UnixSocket` | 多客户端 UDS: `$CODEX_HOME/app-server-control/` |
| `WebSocket` | 多客户端 `ws://IP:PORT` + 令牌认证 |
| `Off` | 无传输监听（仅远程控制路径） |

### 双循环架构

```
Transport → TransportEvent → Processor Task (主事件循环)
                           → Outbound Router Task (写路由，独立)
```

分离消息处理和出站写入，防止慢网络客户端阻塞主处理器。

### JSON-RPC 处理器

- `CodexMessageProcessor` — 核心 Agent 逻辑
- `ConfigApi` — 配置读写
- `FsApi` — 文件系统操作
- `DeviceKeyApi` — 设备密钥签名
- `FsWatchManager` — 文件系统监视

### 优雅关闭

两阶段关闭协议：
1. 第一次信号: 标记关闭，停止接受新连接，等待运行中的 assistant turn 完成
2. 第二次信号: 强制立即关闭

---

## codex-tui — 终端 UI

**路径**: `codex-rs/tui/` | **二进制名**: `codex-tui`

基于 **Ratatui** + **crossterm** 的丰富终端交互界面。

### 运行模式

| 模式 | 说明 |
|------|------|
| `Embedded` | 进程内启动 app-server（默认） |
| `Remote` | 连接到远程 WebSocket app-server |

### 核心组件

| 组件 | 文件 | 说明 |
|------|------|------|
| `App` | `src/app.rs` | 顶级应用状态机 |
| `ChatWidget` | - | 主对话渲染 |
| `CwdPrompt` | - | 工作目录选择 |
| `ResumePicker` | - | 会话历史选择器 |
| `ExternalEditor` | - | 外部编辑器提示输入 |
| `VoiceCapture` | - | 麦克风输入（macOS/Windows） |
| `RealtimeAudioPlayer` | - | WebRTC 音频输出 |
| `FileSearchManager` | - | Tab 补全模糊文件搜索 |
| `McpServerElicitation` | - | MCP 服务器权限提示 |

### 渲染特性

- **Markdown 渲染**: 流式输出、语法高亮（syntect）
- **Diff 渲染**: 彩色添加/删除行
- **动画**: ASCII 动画和 shimmer 效果
- **桌面通知**: 终端响铃、标题变更、系统通知
- **内联模式**: 保留终端滚动历史（非全屏模式）
- **Zellij 支持**: 特殊 scrollback 处理

---

## codex-mcp-server — MCP 服务器

**路径**: `codex-rs/mcp-server/` | **二进制名**: `codex-mcp-server`

通过 **Model Context Protocol** (MCP) 暴露 Codex 能力，使用 rmcp crate，自定义 stdio 传输。

### 协议处理

| MCP 方法 | 说明 |
|----------|------|
| `initialize` | 返回服务器能力（tools 支持） |
| `tools/list` | 返回完整工具目录 |
| `tools/call` | 执行工具调用（含审批流程） |
| `resources/list` / `resources/read` | 资源访问 |
| `prompts/list` / `prompts/get` | 提示模板 |
| `ping` | 心跳 |

### 审批流程

自定义带外审批请求：危险操作（shell 命令、文件补丁）通过通知发送审批请求，客户端通过特殊 `codex_tool_call_reply` 工具调用响应。

### 架构要点

- 所有日志输出到 stderr（不干扰 stdout 上的 JSON-RPC）
- 暴露 Codex 的完整工具目录（非子集）
- 使用 `ThreadManager` 创建和管理 Agent 线程

---

## codex-exec — 非交互执行

**路径**: `codex-rs/exec/` | **二进制名**: `codex-exec`

连接 app-server 并处理执行事件，输出人类可读或 JSONL 格式的结果。

### 输出模式

| 模式 | 标志 | 说明 |
|------|------|------|
| 人类可读 | 默认 | 彩色 stderr 进度 + stdout 最终消息 |
| JSONL | `--json` | 结构化 JSONL 事件流到 stdout |

### JSONL 事件类型

`thread.started`, `thread.completed`, `turn.started`, `turn.completed`, `item.started` (agent_message, file_change, command_execution, reasoning, web_search, mcp_tool_call), `item.updated`, `item.completed`, `error`

### 子命令

- **Resume**: 通过 ID 或 `--last` 恢复之前会话
- **Review**: 对未提交更改、基准分支或特定提交进行代码审查

### 架构要点

- 启动进程内 app-server（共享核心 Agent 逻辑）
- 审批策略默认 `Never`（无用户在场）；功能标志可覆盖
- 兼容 CI/CD 管道（`--json` + `--output-last-message`）

---

## codex-file-search — 文件搜索

**路径**: `codex-rs/file-search/` | **二进制名**: `codex-file-search`

基于 **nucleo** 匹配引擎的模糊文件路径搜索工具。用于 Tab 补全和交互式文件选择。
