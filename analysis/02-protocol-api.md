# 协议与 API 层

涵盖 9 个 crate：`codex-protocol`, `codex-app-server-protocol`, `codex-api`, `codex-client`, `codex-codex-mcp`, `codex-rmcp-client`, `codex-backend-openapi-models`, `codex-backend-client`, `codex-codex-experimental-api-macros`

---

## codex-protocol — 核心协议类型

**路径**: `codex-rs/protocol/`

Codex 中使用的共享协议类型和模型定义。

### 核心类型

| 类型 | 说明 |
|------|------|
| `ThreadId` | 会话线程唯一标识符 |
| `ToolName` | 工具名称标识符 |
| `AgentPath` | Agent 多路径标识 |
| 审批类型 | 执行审批、补丁审批、MCP 审批 |
| 权限类型 | 权限配置文件和沙箱策略 |
| 模型类型 | 模型信息、预设、配置 |
| Exec 输出 | 进程输出块、退出状态 |
| 网络策略 | 网络规则和限制 |
| 账户类型 | 提供商账户和认证状态 |
| 错误类型 | 统一错误处理和映射 |

---

## codex-app-server-protocol — App Server 协议

**路径**: `codex-rs/app-server-protocol/`

App Server 的 JSON-RPC v1/v2 协议定义。

### 协议版本

| 版本 | 说明 |
|------|------|
| V1 | 初始协议，包含请求/响应/通知类型的完整模式定义 |
| V2 | 更新版本，支持实验性 API 定义，TypeScript/JSON Schema 生成 |

### 消息类型

- **请求**: `initialize`, `thread/start`, `thread/resume`, `turn/start`, `turn/interrupt`, `turn/steer` 等
- **响应**: 操作结果、错误信息
- **通知**: `thread/started`, `turn/completed`, `item/started`, `item/updated`, `item/completed` 等 30+ 通知类型

### 代码生成

- `codex-app-server-protocol` 支持生成 TypeScript 绑定和 JSON Schema
- `codex-experimental-api-macros` 提供过程宏用于定义实验性 API

---

## codex-api — 高层 API 客户端

**路径**: `codex-rs/codex-api/` | **规模**: 30+ 源文件

高层 OpenAI/Codex 后端 API 客户端。

### 端点客户端

| 客户端 | 端点 | 说明 |
|--------|------|------|
| `ResponsesClient` | `POST /responses` | Responses API 流式请求（SSE） |
| `ResponsesWebsocketClient` | `wss://.../v1/responses` | WebSocket 传输（HTTP 回退） |
| `ModelsClient` | `GET /models` | 模型目录获取（ETag 支持） |
| `CompactClient` | `POST /responses/compact` | 对话压缩 |
| `MemoriesClient` | `POST /memories/trace_summarize` | 记忆摘要 |
| `RealtimeWebsocketClient` | WebSocket | 实时音频流式 |
| File Upload | `POST /files` → PUT Blob | 三步文件上传 |

### SSE 流式处理

```rust
ResponsesClient::stream_request()
  → EndpointSession::stream_with()
    → POST with auth + retries
      → spawn_response_stream()
        → process_sse() [idle timeout 循环]
          → process_responses_event()
            → ResponseEvent 分发
```

### `ResponseEvent` 枚举

`Created`, `OutputItemDone`, `OutputItemAdded`, `ServerModel`, `ModelVerifications`, `ServerReasoningIncluded`, `Completed { response_id, token_usage }`, `OutputTextDelta`, `ToolCallInputDelta`, `ReasoningSummaryDelta`, `ReasoningContentDelta`, `ReasoningSummaryPartAdded`, `RateLimits`, `ModelsEtag`

### 错误映射

`map_api_error()` 将 `ApiError` 转换为 `CodexErr`：
- 503 + `server_is_overloaded` → 服务器过载
- 400 + cyber policy → 内容策略违规
- 429 → 使用量限制
- 上下文窗口超出 → 自动压缩触发
- 提取调试头: `cf-ray`, `x-request-id`, `x-oai-request-id`, `x-error-json`

---

## codex-client — 低层 HTTP 传输

**路径**: `codex-rs/codex-client/`

低层 HTTP 传输客户端：

- 管理 `reqwest` 连接池和配置
- 自定义 CA 证书支持
- 重试策略（指数退避 + jitter）
- SSE 解析（逐行读取 + 事件分发）
- 请求遥测（耗时、状态码、错误）
- HTTP 传输抽象层

---

## codex-codex-mcp — MCP 集成层

**路径**: `codex-rs/codex-mcp/`

MCP（Model Context Protocol）集成层，管理 MCP 服务器连接：

- **连接管理**: MCP 服务器生命周期（连接、断开、重连）
- **OAuth 登录**: MCP OAuth 认证流程
- **工具来源**: 追踪工具来源（内置 vs MCP 服务器）
- **快照收集**: MCP 服务器状态快照

---

## codex-rmcp-client — rmcp 客户端

**路径**: `codex-rs/rmcp-client/`

基于 rmcp crate 的 MCP 客户端实现：

- **OAuth 流程**: 完整的 MCP OAuth 2.0 授权流程
- **stdio 服务器启动**: 启动和管理 MCP 服务器子进程
- **提取 (Elicitation)**: 权限请求提取
- **执行器进程传输**: 通过进程通信传输 MCP 消息

---

## codex-backend-openapi-models — 后端 OpenAPI 模型

**路径**: `codex-rs/codex-backend-openapi-models/`

从后端 API 规范自动生成的 OpenAPI 模型：
- 自动生成的 serde 类型（无手写代码）
- 与后端 API 规范保持同步

---

## codex-backend-client — 后端 API 客户端

**路径**: `codex-rs/backend-client/`

Codex 云服务的后端 API 客户端：
- 获取任务详情（`/wham/tasks/{task_id}`）
- 获取配置文件
- 获取 turn 数据
- 与 ChatGPT 后端交互

---

## codex-codex-experimental-api-macros — 实验性 API 宏

**路径**: `codex-rs/codex-experimental-api-macros/`

过程宏 crate，用于定义实验性 API：
- 由 `codex-app-server-protocol` 使用
- 自动生成序列化/反序列化代码
- 管理实验性 API 版本标记
