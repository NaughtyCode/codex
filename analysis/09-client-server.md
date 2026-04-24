# 客户端/服务器通信

涵盖 3 个 crate：`codex-app-server-client`, `codex-app-server-test-client`, `codex-debug-client`

---

## codex-app-server-client — App Server 客户端

**路径**: `codex-rs/app-server-client/`

连接 app-server 的客户端，通过进程内通道或 WebSocket 连接。

### 连接模式

| 模式 | 说明 |
|------|------|
| **进程内 (In-Process)** | 直接创建 app-server 实例（共享内存通信） |
| **WebSocket** | 通过 `ws://` 或 `wss://` 连接到远程 app-server |

### 服务生命周期

- **启动**: 创建和初始化 app-server 实例
- **环境管理**: 配置执行环境（本地/远程）
- **连接管理**: Websocket 握手、认证、重连
- **关闭**: 优雅关闭和资源清理

### 核心功能

```rust
// 连接流程
InProcessAppServerClient::connect(args)
  → 创建或连接 app-server
    → 建立 JSON-RPC 通道
      → 发送请求 / 接收事件
```

---

## codex-app-server-test-client — 测试客户端

**路径**: `codex-rs/app-server-test-client/`

用于手动/交互式测试的 app-server 测试工具客户端。

### 功能

- 通过 WebSocket 连接到 app-server
- 发送测试 JSON-RPC 请求
- 接收和显示响应/通知
- 用于手动测试和调试协议交互

---

## codex-debug-client — 调试客户端

**路径**: `codex-rs/debug-client/`

带有 CLI 的调试客户端，用于测试 app-server 协议交互。

### 功能

- 交互式调试 CLI
- 发送原始 JSON-RPC 消息
- 检查响应和通知
- 协议版本测试
- App Server 功能探索

---

## 通信架构

```
┌─────────────────────────────┐
│ TUI / CLI / Exec Binary     │
│  (app-server-client)        │
│    ├── InProcess → app-server (直接内存通信)
│    └── WebSocket → app-server (ws:// 或 wss://)
└──────────────┬──────────────┘
               │ JSON-RPC 2.0
               ▼
┌─────────────────────────────┐
│ codex-app-server            │
│  ├── Processor Task         │
│  ├── Outbound Router Task   │
│  └── Transport (stdio/UDS/WS)│
└─────────────────────────────┘
               ▲
               │ JSON-RPC 2.0
┌──────────────┴──────────────┐
│ Python / TypeScript SDK     │
│  (app-server-client)        │
│    └── stdio → exec --experimental-json | app-server
└─────────────────────────────┘
```
