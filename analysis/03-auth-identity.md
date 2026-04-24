# 认证与身份

涵盖 6 个 crate：`codex-login`, `codex-agent-identity`, `codex-device-key`, `codex-keyring-store`, `codex-aws-auth`, `codex-otel`

---

## codex-login — 认证管理

**路径**: `codex-rs/login/`

所有认证流程的管理中心。

### 认证方式

| 方式 | 说明 |
|------|------|
| **OAuth 设备码** | 标准 OAuth 2.0 Device Authorization Grant |
| **API Key** | 直接 API key 认证（stdin 输入） |
| **ChatGPT Enterprise** | ChatGPT 企业账户 OAuth 流程 |

### 功能

- **Token 刷新**: 自动刷新过期的访问令牌
- **持久化**: 将凭据安全保存到磁盘
- **多重认证状态**: API key / ChatGPT 账户 / 未认证
- **认证来源追踪**: 记录凭据来源（环境变量、keyring、配置文件）

### 认证流程

```
CLI 触发 login 子命令
  → 选择认证方式
    ├── API Key: 从 stdin 读取 → 验证 → 持久化
    ├── OAuth 设备码: 启动设备流程 → 显示验证码 → 轮询 token → 持久化
    └── ChatGPT: OAuth 流程 → 获取账户 token → 持久化
  → 认证就绪
```

---

## codex-agent-identity — Agent 身份密钥

**路径**: `codex-rs/agent-identity/`

Agent 身份密钥管理，用于委托 Agent 执行。

### 密钥类型

| 密钥 | 算法 | 用途 |
|------|------|------|
| 签名密钥 | Ed25519 | 签名 Agent 断言和请求 |
| 密钥协商 | Curve25519 | 加密密钥协商 |

### 功能

- **Agent 注册**: 向服务注册 Agent 身份
- **断言生成**: 创建和签署 Agent 断言
- **委派执行**: 代表用户执行授权的 Agent 操作

---

## codex-device-key — 设备密钥

**路径**: `codex-rs/device-key/`

设备绑定密钥基础设施，用于远程控制会话绑定。

### 技术细节

- **算法**: P-256 ECDSA
- **存储**: 硬件支持或操作系统保护的密钥存储
- **用途**: 在远程控制场景中验证设备身份

---

## codex-keyring-store — 凭据存储

**路径**: `codex-rs/keyring-store/`

平台原生凭据存储抽象层，封装 `keyring` crate。

### 平台后端

| 平台 | 后端 |
|------|------|
| macOS | **Keychain** (Security.framework) |
| Windows | **Credential Manager** (CredRead/CredWrite) |
| Linux | **secret-service** (org.freedesktop.secrets) |

### API

- `get_password(service, username)` / `set_password(service, username, password)`
- 跨平台统一接口
- 错误处理（后端不可用时的回退策略）

---

## codex-aws-auth — AWS 认证

**路径**: `codex-rs/aws-auth/`

AWS SigV4 请求签名，用于 Amazon Bedrock 集成。

### 功能

- **凭据解析**: 从环境变量、AWS 配置文件、IAM 角色解析 AWS 凭据
- **SigV4 签名**: 对 HTTP 请求应用 AWS Signature Version 4
- **区域管理**: 支持 12 个 AWS 区域
- **Mantle 端点**: 构造 Bedrock Mantle 端点 URL

---

## codex-otel — OpenTelemetry 集成

**路径**: `codex-rs/otel/`

全栈可观测性集成。

### 功能

| 领域 | 说明 |
|------|------|
| **Metrics** | 请求耗时、token 用量、工具调用频率等 |
| **Tracing** | 分布式追踪，span 生成和传播 |
| **OTLP Export** | 通过 OpenTelemetry Protocol 导出遥测数据 |
| **Runtime Metrics** | Tokio 运行时统计、内存使用 |
| **Session Telemetry** | 会话级别遥测（turn 计数、压缩事件等） |
| **Timer 工具** | 高精度计时代码路径 |

### 架构

```
Tracing subscriber (tracing-subscriber)
  → OpenTelemetry layer
    → OTLP exporter → 后端收集器
  → File layer (codex-tui.log)
  → SQLite layer (codex-state)
```
