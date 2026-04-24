# 核心 Agent 引擎

涵盖 8 个 crate：`codex-core`, `codex-thread-store`, `codex-rollout`, `codex-rollout-trace`, `codex-state`, `codex-hooks`, `codex-features`, `codex-config`

---

## codex-core — 中心引擎

**路径**: `codex-rs/core/` | **类型**: lib crate | **规模**: ~230+ 源文件

整个 Codex Agent 系统的核心编排层。包含 Agent 循环、会话管理、上下文构建、工具调度、插件/技能加载、MCP 工具管理、执行策略、记忆子系统和配置加载。

### 核心架构类型

| 类型 | 文件 | 说明 |
|------|------|------|
| `Codex` | `session/mod.rs` | 高层 Agent 接口，`Codex::spawn()` 创建 |
| `Session` | `session/session.rs` | 持有线程的所有可变状态（历史、工具、服务） |
| `SessionConfiguration` | `session/session.rs` | 会话设置的不可变快照 |
| `TurnContext` | `session/turn_context.rs` | 每轮配置快照（模型、工具、技能、环境） |
| `CodexThread` | `codex_thread.rs` | 公共 API 封装，`submit()`, `next_event()`, `steer_input()` |
| `ThreadManager` | `thread_manager.rs` | 线程生命周期（创建、恢复、分叉、关闭） |

### Agent 循环流程

```
用户输入 (Op::UserInput)
  → ThreadManager → Codex::spawn()
       → submission_loop() → 事件驱动循环
            → run_turn() → 核心 Agent 循环:
                1. 预采样压缩
                2. 技能/插件注入解析
                3. Hook: user_prompt_submit
                4. 循环: run_sampling_request()
                     → ToolRouter 构建
                     → ModelClientSession::stream()
                     → 工具调用分发与执行
                     → Guardian 自动审查
                5. 自动压缩（token 超限）
                6. Hook: stop → after-agent
                7. 发送完成事件
```

### 工具系统

| 类型 | 文件 | 说明 |
|------|------|------|
| `ToolRouter` | `tools/router.rs` | 构建工具注册表和规格 |
| `ToolRegistry` | `tools/registry.rs` | ToolName → ToolHandler 映射 |
| `ToolHandler` trait | `tools/registry.rs` | 工具执行接口：`kind()`, `handle()`, `is_mutating()` |
| `ToolCallRuntime` | `tools/parallel.rs` | 并行工具执行管理器 |

工具处理器: `shell.rs`, `mcp.rs`, `plan.rs`, `dynamic.rs`, `js_repl.rs`, `apply_patch.rs`, `tool_search.rs`, `tool_suggest.rs`, `multi_agents*.rs`, `unified_exec.rs`

### 记忆子系统

两阶段管道：
- **Phase 1** (`memories/phase1.rs`): 从 rollout 中选择并提取原始记忆
- **Phase 2** (`memories/phase2.rs`): 通过 LLM Agent 将原始记忆整合为结构化记忆

### Guardian 自动审查

独立的 LLM Agent 审查系统：
- 对 shell 命令、补丁应用、MCP 工具调用的自动审查
- `GuardianReviewSessionManager` 管理并发审查会话
- 结果：自动批准 / 拒绝 / 升级给用户

### 配置加载

分层配置栈（优先级从高到低）：
1. Cloud requirements（组织策略）
2. Admin/MDM（macOS 管理设备）
3. System（`/etc/codex/`）
4. User（`$CODEX_HOME/config.toml`）
5. Project（`.codex/config.toml`）
6. Repo（git 根目录 `.codex/config.toml`）
7. Runtime（CLI 参数 / UI 选择）

---

## codex-thread-store — 线程持久化

**路径**: `codex-rs/thread-store/`

存储无关的线程持久化接口：
- **本地实现**: 文件系统存储，支持 write-ahead 持久化和 flush barriers
- **远程实现**: RPC 支持的远程存储

核心抽象: `LiveThread` — 处理线程事件的前向持久化

---

## codex-rollout — Rollout 管理

**路径**: `codex-rs/rollout/`

磁盘上 session 文件的发现和管理：
- 列出 rollout 文件及其元数据
- Rollout 策略配置
- 会话录制和重放

---

## codex-rollout-trace — Trace Bundle

**路径**: `codex-rs/rollout-trace/`

Trace bundle 格式定义：
- 写入器和缩减器
- Trace schema，用于重放和分析 Agent 执行过程
- 支持 JSONL 格式的 rollout 数据提取

---

## codex-state — SQLite 状态存储

**路径**: `codex-rs/state/`

SQLite 支持的状态存储：
- 将 JSONL rollout 数据提取到本地 SQLite 数据库
- 支持高效的线程和 Agent 作业查询
- 管理 rollout 元数据

---

## codex-hooks — Hook 引擎

**路径**: `codex-rs/hooks/`

生命周期 Hook 执行引擎，在关键事件点运行用户配置的 Hook：

| Hook 事件 | 触发时机 |
|-----------|----------|
| `pre-tool-use` | 工具调用执行前 |
| `post-tool-use` | 工具调用执行后 |
| `session-start` | 会话启动时 |
| `stop` | 会话停止时 |
| `user-prompt-submit` | 用户提交提示时 |

Hook 可以注入额外上下文、修改行为或执行自定义操作。

---

## codex-features — 功能标志

**路径**: `codex-rs/features/`

集中式功能标志注册中心：

- **Stage 枚举**: `UnderDevelopment`（开发中）、`Experimental`（实验性）、`GeneralAvailability`（正式发布）
- **标志解析**: 从配置中解析有效功能集
- **管理**: 支持通过 CLI（`codex features`）或 API 启用/禁用功能

---

## codex-config — 配置管理

**路径**: `codex-rs/config/`

配置加载和解析：
- `config.toml` — 主配置文件
- `permissions.toml` — 权限配置
- `profiles.toml` — 配置文件（环境预设）
- 云需求约束处理
- 配置合并与验证
- 配置 schema 生成（`config_schema.rs`）
