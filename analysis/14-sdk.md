# SDK 包

涵盖 3 个 SDK 包：TypeScript SDK、Python SDK、Python Runtime。

---

## 1. TypeScript SDK (`@openai/codex-sdk`)

**路径**: `sdk/typescript/`
**包名**: `@openai/codex-sdk`
**构建**: pnpm + tsup (ESM-only, Node 18+)

### 用途

将 Codex CLI Agent 嵌入 TypeScript/Node.js 工作流：生成 CLI 子进程，通过 stdin/stdout 交换 JSONL 事件。

### 架构

```
用户代码
  ↓ import { Codex, Thread }
  ↓
Codex.startThread() → CodexExec.spawn()
  ↓ 生成 codex exec --experimental-json
  ↓ JSONL over stdin/stdout
  ↓
事件循环 → ThreadEvent (type union)
```

### 核心 API

| 类/类型 | 文件 | 说明 |
|---------|------|------|
| `Codex` | `codex.ts` | 主入口 — `startThread()`, `resumeThread()` |
| `Thread` | `thread.ts` | 线程生命周期 — `run()`, `runStreamed()` |
| `CodexOptions` | `codexOptions.ts` | 配置：`codexPathOverride`, `baseUrl`, `apiKey`, `config`, `env` |
| `ThreadOptions` | `threadOptions.ts` | `SandboxMode`, `ApprovalMode`, `ModelReasoningEffort`, `WebSearchMode` |
| `TurnOptions` | `turnOptions.ts` | `outputSchema` (JSON Schema), `signal` (AbortSignal) |

### 事件系统

**`ThreadEvent`** 联合类型:
`thread.started`, `turn.started`, `turn.completed`, `turn.failed`, `item.started`, `item.updated`, `item.completed`, `error`

**`ThreadItem`** 联合类型:
`AgentMessageItem`, `ReasoningItem`, `CommandExecutionItem`, `FileChangeItem`, `McpToolCallItem`, `WebSearchItem`, `TodoListItem`, `ErrorItem`

### CLI 二进制解析

使用 `@openai/codex` npm 包和平台特定可选依赖包（`@openai/codex-linux-x64` 等）在 macOS、Linux、Windows 上 x64/arm64 运行时定位原生二进制。

### 示例

- `basic_streaming.ts` — 基本流式输出 demo
- `structured_output.ts` — JSON Schema 结构化输出
- `structured_output_zod.ts` — Zod schema 结构化输出

---

## 2. Python SDK (`codex-app-server-sdk`)

**路径**: `sdk/python/`
**包名**: `codex-app-server-sdk`
**版本**: `0.2.0`
**构建**: Hatchling, Python ≥ 3.10, pydantic ≥ 2.12

### 用途

面向 `codex app-server` JSON-RPC v2 协议的实验性 Python SDK（stdio 传输），提供同步和异步客户端。

### 架构

```python
from codex_app_server import Codex, Thread

# 同步
async with Codex() as codex:
    thread = codex.thread_start(...)
    handle = thread.turn(...)
    result = handle.wait()

# 异步
async with AsyncCodex() as codex:
    thread = await codex.thread_start(...)
    ...
```

### 核心模块

| 模块 | 文件 | 说明 |
|------|------|------|
| `AppServerClient` | `client.py` | 核心同步 JSON-RPC v2 客户端（管理子进程生命周期） |
| `AsyncAppServerClient` | `async_client.py` | 异步封装（使用 `asyncio.to_thread` + `asyncio.Lock`） |
| `Codex` / `AsyncCodex` | `api.py` | 高层 API：`run()` 简化一次性 turn，`turn()` 完整控制 |
| `Thread` / `AsyncThread` | `api.py` | 线程生命周期方法 |
| `TurnHandle` / `AsyncTurnHandle` | `api.py` | Turn 流式、转向、中断控制 |

### 通知类型 (30+)

`NotificationPayload` 联合类型：`thread/started`, `turn/completed`, `item/started`, `item/updated`, `item/completed` 等。

### 错误层次

```
AppServerError
  → JsonRpcError
    → AppServerRpcError
      ├── ParseError
      ├── InvalidRequestError
      ├── MethodNotFoundError
      ├── InvalidParamsError
      ├── InternalRpcError
      └── ServerBusyError
  → TransportClosedError
```

### 生成代码

- `generated/v2_all.py` (7765 行) — 从 app-server v2 协议 schema 自动生成的 Pydantic 模型
- `generated/notification_registry.py` — 通知方法映射到模型类

### 14 个示例

从基本用法（构造、turn、流式）到高级场景（模型选择、错误重试、CLI mini-app）。

---

## 3. Python Runtime (`openai-codex-cli-bin`)

**路径**: `sdk/python-runtime/`
**包名**: `openai-codex-cli-bin`
**构建**: Hatchling + 自定义 `RuntimeBuildHook`

### 用途

平台特定运行时包，被 `codex-app-server-sdk` 使用。携带原生 `codex` 二进制，使 SDK 可以精确固定 CLI 版本，无需将二进制检入仓库。仅发布 wheel（无 sdist）。

### 二进制定位

```python
# bundled_codex_path() 在 __file__/../bin/codex (或 .exe) 定位打包的二进制
```

### 自定义 Hatch 构建 Hook

- 生成平台特定 wheels（`pure_python=False`）
- 标签如 `py3-none-manylinux_2_28_x86_64`
- 拒绝 sdist 构建
