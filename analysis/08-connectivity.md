# 网络与连接

涵盖 6 个 crate：`codex-connectors`, `codex-network-proxy`, `codex-cloud-requirements`, `codex-cloud-tasks`, `codex-cloud-tasks-client`, `codex-cloud-tasks-mock-client`

---

## codex-connectors — 连接器集成

**路径**: `codex-rs/connectors/`

ChatGPT 连接器（应用）发现和缓存管理。

### 功能

- **发现**: 从 ChatGPT 目录 API 获取可用连接器列表
- **缓存**: 缓存连接器信息以减少 API 调用
- **合并**: 合并目录连接器与 MCP 工具发现的连接器
- **品牌信息**: 连接器品牌、图标和元数据管理

### 连接器来源

```
list_connectors()
  ├── 目录列出的连接器 (ChatGPT directory API)
  ├── MCP 工具发现的可访问连接器
  └── 插件应用连接器
```

---

## codex-network-proxy — 网络代理服务器

**路径**: `codex-rs/network-proxy/`

网络代理服务器实现，用于透明地路由受限网络的沙箱流量。

### 代理协议

| 协议 | 说明 |
|------|------|
| **HTTP CONNECT** | HTTP CONNECT 隧道 |
| **SOCKS5** | SOCKS5 代理 |
| **MITM** | Man-in-the-Middle 代理（用于检查和修改） |

### 功能

- **网络策略执行**: 基于策略允许/拒绝出站连接
- **上游代理支持**: 将流量转发到配置的上游代理
- **域名解析**: DNS 解析和 IP 过滤

---

## codex-cloud-requirements — 云需求

**路径**: `codex-rs/cloud-requirements/`

从后端获取 `requirements.toml` 文件：

- Business/Enterprise ChatGPT 账户的云托管配置要求
- 组织级别的策略配置（模型限制、功能标志、安全策略）
- 作为配置加载栈的最高优先级层

---

## codex-cloud-tasks — 云任务 TUI

**路径**: `codex-rs/cloud-tasks/`

云任务管理终端 UI：
- 列出云托管的代码审查任务
- 显示 diff 渲染
- 任务状态管理

---

## codex-cloud-tasks-client — 云任务客户端

**路径**: `codex-rs/cloud-tasks-client/`

云任务后端 API 的客户端：

### 核心 Trait

```rust
trait CloudBackend {
    // 任务 CRUD 操作
    async fn list_tasks(...);
    async fn get_task(...);
    async fn update_task(...);

    // 连接管理
    async fn connect(...);
}
```

- HTTP 客户端实现
- 认证集成（重用 Agent 登录凭据）

---

## codex-cloud-tasks-mock-client — 云任务 Mock

**路径**: `codex-rs/cloud-tasks-mock-client/`

`CloudBackend` trait 的 Mock 实现，用于测试：
- 模拟任务 CRUD 响应
- 用于集成测试和 UI 开发
