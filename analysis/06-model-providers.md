# 模型提供商与 AI 集成

涵盖 9 个 crate：`codex-model-provider-info`, `codex-model-provider`, `codex-models-manager`, `codex-chatgpt`, `codex-lmstudio`, `codex-ollama`, `codex-realtime-webrtc`, `codex-response-debug-context`, `codex-responses-api-proxy`

---

## codex-model-provider-info — 提供商注册表

**路径**: `codex-rs/model-provider-info/`

### 核心类型: `ModelProviderInfo`

| 字段 | 说明 |
|------|------|
| `name` | 显示名称 |
| `base_url` | OpenAI 兼容 API 端点 URL |
| `env_key` | API key 环境变量名 |
| `auth` (optional) | 命令支持的认证配置 |
| `aws` (optional) | AWS SigV4 认证配置 |
| `wire_api` | 线路协议（仅 `Responses`） |
| `query_params` | 额外查询参数 |
| `http_headers` / `env_http_headers` | 静态和环境变量 HTTP 头 |
| `request_max_retries` / `stream_max_retries` | 重试配置 |
| `requires_openai_auth` | 是否需要 OpenAI 登录 |
| `supports_websockets` | 是否支持 WebSocket |

### 内置提供商

| ID | 名称 | 特点 |
|----|------|------|
| `openai` | OpenAI | 需要 OAuth 登录，支持 WebSocket，注入版本/组织/项目头 |
| `amazon-bedrock` | Amazon Bedrock | SigV4 签名，Mantle 端点，12 区域支持 |
| `ollama` | Ollama | 本地 `http://localhost:11434/v1` |
| `lmstudio` | LM Studio | 本地 `http://localhost:1234/v1` |

### 验证规则

- `aws` 与 `env_key`、`experimental_bearer_token`、`auth` 互斥
- `supports_websockets` 与 AWS auth 不兼容
- 内置提供商不可覆盖（除 Bedrock 的 `aws.profile`/`aws.region`）

---

## codex-model-provider — 提供商抽象

**路径**: `codex-rs/model-provider/`

### 核心 Trait: `ModelProvider`

```rust
trait ModelProvider {
    fn info() -> ModelProviderInfo;
    fn auth_manager() -> Option<AuthManager>;
    async fn auth() -> Result<CodexAuth>;
    fn account_state() -> ProviderAccountState;
    fn api_provider() -> Provider;
    fn api_auth() -> SharedAuthProvider;
    fn models_manager() -> Arc<dyn ModelsManager>;
}
```

### 认证策略

| 策略 | 说明 |
|------|------|
| Bearer Token | 从 env var、token 文件或 CodexAuth 获取 |
| Agent Identity | 使用私钥签名请求 |
| Unauthenticated | 无操作（本地 OSS 提供商） |
| Command-backed | 运行外部命令获取 token |
| AWS SigV4 | IAM 签名（Amazon Bedrock） |

### Amazon Bedrock

- `BedrockMantleSigV4AuthProvider`: 应用 AWS SigV4 签名
- 支持 12 个 AWS 区域
- 静态模型目录: `gpt-5.4-cmb`（默认）、`gpt-oss-120b`、`gpt-oss-20b`

---

## codex-models-manager — 模型目录管理

**路径**: `codex-rs/models-manager/`

### 核心功能

1. **发现**: 从 `models.json` 加载捆绑预设 + 从远程 API 获取
2. **缓存**: 磁盘缓存到 `{codex_home}/models_cache.json`（TTL 300s）
3. **解析**: 最长前缀匹配 → 命名空间后缀匹配 → 回退最小描述符
4. **覆盖**: 配置覆盖（上下文窗口、截断策略、基础指令）

### 刷新策略

| 策略 | 说明 |
|------|------|
| `Online` | 始终从网络获取 |
| `Offline` | 仅使用缓存 |
| `OnlineIfUncached` | 缓存优先，未命中时回退网络 |

### 模型信息解析

```
construct_model_info_from_candidates():
  1. 最长前缀匹配远程 slug
  2. 命名空间后缀匹配 (e.g., custom/gpt-5.3-codex)
  3. 回退 model_info_from_slug() (128K context, 无 reasoning)
  4. 应用 with_config_overrides()
```

---

## codex-api — OpenAI API 客户端

**路径**: `codex-rs/codex-api/` | **规模**: 30+ 源文件

### 传输层

- **HTTP SSE 流式**: `POST /responses` → `ResponseStream`
- **WebSocket**: `wss://.../v1/responses` → `ResponsesWebsocketClient`
- **文件上传**: 三步协议（请求 URL → PUT Azure Blob → 完成回调）
- **模型列表**: `GET /models` + ETag 条件请求
- **压缩**: `POST /responses/compact`
- **记忆摘要**: `POST /memories/trace_summarize`

### 流式处理 (`sse/responses.rs`)

SSE 事件映射到 `ResponseEvent`：
- `response.created` → `Created`
- `response.output_item.done` → `OutputItemDone`
- `response.output_text.delta` → `OutputTextDelta`
- `response.completed` → `Completed { response_id, token_usage }`
- `response.failed` → 具体 `ApiError` 变体

### 速率限制

解析 `x-{limit_id}-primary-*` 和 `x-{limit_id}-secondary-*` 响应头，支持多限制族（`codex`, `codex_secondary`, `codex_bengalfox`）。

---

## codex-chatgpt — ChatGPT 集成

**路径**: `codex-rs/chatgpt/`

ChatGPT 后端 API 客户端：
- 任务获取与 diff 应用 (`/wham/tasks/{task_id}`)
- 连接器（应用）发现和管理
- 工作区可访问性合并

---

## codex-lmstudio — LM Studio 集成

**路径**: `codex-rs/lmstudio/`

本地 LM Studio 推理服务器管理：
- 服务器探测: `GET /models`
- 模型加载: 发送 `max_output_tokens: 1` 最小请求
- 模型下载: shell out to `lms get --yes {model}`
- 二进制发现: PATH → `~/.lmstudio/bin/lms` 回退
- 默认模型: `openai/gpt-oss-20b`

---

## codex-ollama — Ollama 集成

**路径**: `codex-rs/ollama/`

本地 Ollama 推理服务器管理：
- OpenAI 兼容端点检测（检测 `/v1` 后缀）
- 版本检查: ≥ 0.13.4 以支持 Responses API
- 模型拉取: 流式 `POST /api/pull` + 进度报告
- CLI/TUI 进度报告器
- 默认模型: `gpt-oss:20b`

---

## codex-realtime-webrtc — WebRTC 实时交互

**路径**: `codex-rs/realtime-webrtc/`

macOS 原生 WebRTC 会话用于语音/实时 AI 交互：
- SDP offer/answer 处理
- 通过原生框架获取音频级别事件

---

## codex-response-debug-context — 调试上下文剥离

**路径**: `codex-rs/response-debug-context/`

从 API 请求头中剥离调试上下文信息用于日志/调试。

---

## codex-responses-api-proxy — 响应 API 代理

**路径**: `codex-rs/responses-api-proxy/`

轻量 HTTP 代理用于 OpenAI Responses API：
- 转发请求并注入 API key
- 用于沙箱化子进程环境，使子进程无需直接持有凭证
