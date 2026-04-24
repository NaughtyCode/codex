# 执行与沙箱

涵盖 9 个 crate：`codex-exec-server`, `codex-arg0`, `codex-sandboxing`, `codex-linux-sandbox`, `codex-windows-sandbox`, `codex-apply-patch`, `codex-shell-command`, `codex-shell-escalation`, `codex-process-hardening`

---

## codex-exec-server — 执行后端

**路径**: `codex-rs/exec-server/` | **规模**: ~30 源文件

整个执行系统的公开 API。管理进程生命周期、文件系统操作、HTTP 代理和环境选择（本地 vs 远程）。作为接受 JSON-RPC 2.0 连接的服务器运行，支持 WebSocket 和 stdio 传输。

### 核心类型

| 类型 | 文件 | 说明 |
|------|------|------|
| `ExecProcess` trait | `process.rs` | 进程抽象：`process_id()`, `read()`, `write()`, `terminate()` |
| `ExecBackend` trait | `process.rs` | `start(ExecParams) → StartedExecProcess` |
| `ExecProcessEvent` | `process.rs` | `Output`, `Exited`, `Closed`, `Failed` |
| `ExecProcessEventLog` | `process.rs` | 双界重放历史（事件计数 + 字节数） |
| `EnvironmentManager` | `environment.rs` | 本地/远程执行环境选择 |
| `ExecutorFileSystem` trait | `file_system.rs` | `read_file`, `write_file`, `create_directory` 等 |

### 文件系统层次

```
LocalFileSystem
├── UnsandboxedFileSystem → DirectFileSystem (tokio::fs)
└── SandboxedFileSystem → FileSystemSandboxRunner → 沙箱化辅助进程
```

### RPC 框架

- `RpcRouter<S>`: 类型安全路由注册 — `request()`, `request_with_id()`, `notification()`
- `RpcClient`: 完整 JSON-RPC 2.0 客户端，支持待处理请求追踪、原子 request ID、断连检测
- `ExecServerClient`: 远程 exec-server 通信的完整客户端实现
- `LocalProcess`: 本地进程生成和生命周期管理（PTY/pipe 模式）
- `RemoteProcess`: 通过 `LazyRemoteExecServerClient` 委托给远程 exec-server

### 本地进程管理

- `ExecProcessEventLog`: broadcast channel 实现 live fan-out
- 退出进程保留 30 秒供输出读取（测试中为 25ms）
- 每进程输出保留: 1MB
- 环境变量策略: `ExecEnvPolicy`（allowlist/blocklist/overlay）

### 连接架构

```
Transport (stdio/WebSocket)
  → ConnectionProcessor → RpcRouter → ExecServerHandler
       → ProcessHandler → LocalProcess → PTY/Pipe → 子进程
       → FileSystemHandler → LocalFileSystem
       → HTTP Request 处理（流式 via reqwest）
```

---

## codex-arg0 — 多名称二进制调度

**路径**: `codex-rs/arg0/`

基于 PATH 的子进程调度机制:
- 通过自引用 PATH 条目实现正确的子进程调度
- `arg0_dispatch_or_else()` 检查 `argv[0]`，在 `codex`、`codex-exec`、`codex-mcp-server`、`codex-linux-sandbox` 等身份间切换
- 在测试和生产中均适用

---

## codex-sandboxing — 沙箱管理器

**路径**: `codex-rs/sandboxing/`

平台无关的沙箱管理器：

| 类型 | 说明 |
|------|------|
| `SandboxType` | `None`, `MacOsSeatbelt`, `LinuxSeccomp`, `WindowsRestrictedToken` |
| `SandboxablePreference` | `Auto`, `Require`, `Forbid` |
| `SandboxManager` | 选择和应用沙箱技术 |
| `SandboxTransformRequest` | 命令 + 策略 + 偏好 → `SandboxExecRequest` |

### macOS Seatbelt (`seatbelt.rs`, 722 行)

- 生成内联 `.sbpl` 配置文件
- 文件读取: 全盘或受限根目录作用域
- 文件写入: 全盘或显式可写根目录 + 只读子路径
- 网络: 动态策略（代理路由/全网络/拒绝）
- Glob 转换: 将 git-style glob 映射到 Seatbelt 正则 deny 规则
- Unix Socket 访问策略

### Linux Landlock/Bubblewrap

- bubblewrap 文件系统隔离 + seccomp 网络过滤
- 双层架构: 外层 bwrap 做 FS 隔离，内层 seccomp 做网络限制

### 策略转换

- `merge_permission_profiles()`: 多源权限联合
- `intersect_permission_profiles()`: 授权条目限制到请求范围
- `effective_file_system_sandbox_policy()`: 有效 FS 沙箱策略
- `effective_network_sandbox_policy()`: 有效网络沙箱策略

---

## codex-linux-sandbox — Linux 沙箱二进制

**路径**: `codex-rs/linux-sandbox/` | **规模**: ~3000+ 行

独立二进制 `codex-linux-sandbox`，实现两阶段 Linux 沙箱：

### 执行流程

```
codex-linux-sandbox [外层]
  → bubblewrap（文件系统挂载构造）
       → 通过 bwrap re-exec
  → codex-linux-sandbox [内层: --apply-seccomp-then-exec]
       → proxy_routing 激活
       → landlock: apply_sandbox_policy_to_current_thread()
            → PR_SET_NO_NEW_PRIVS
            → seccomp BPF 过滤器
       → execvp(目标命令)
```

### Bubblewrap (`bwrap.rs`, 2022 行)

挂载顺序（关键正确性要求）：
1. 读取挂载: `--ro-bind / /` 或 `--tmpfs /` + 作用域 `--ro-bind`
2. `--dev /dev` 最小设备节点
3. 不可读祖先目录掩码
4. `--bind <root> <root>` 可写根目录
5. 可写根目录下只读子路径重新应用
6. 嵌套不可读挖除
7. 不可读路径掩码

符号链接感知: 解析符号链接可写根目录到规范目标，阻止 writeable symlink 跨越

### Seccomp (`landlock.rs`)

| 模式 | 说明 |
|------|------|
| `Restricted` | 拒绝 connect, accept, bind, listen, sendto, ptrace, io_uring_* 等；仅允许 AF_UNIX socket |
| `ProxyRouted` | 允许 AF_INET/AF_INET6 socket；拒绝 AF_UNIX socketpair |

### 代理路由 (`proxy_routing.rs`, 798 行)

双进程桥接架构：
- **HostBridge**: 宿主机 UDS 监听器 → TCP 到真实代理
- **LocalBridge**: 沙箱内本地 TCP 监听器 → UDS 到宿主机桥接

---

## codex-windows-sandbox — Windows 沙箱

**路径**: `codex-rs/windows-sandbox-rs/`

单文件实现，使用：
- **Restricted Tokens**: 移除管理员权限和危险特权的受限令牌
- **Capability SIDs**: 通过 capability SID 控制细粒度权限
- **ACL 修改**: 允许/拒绝 ACE 条目
- **ConPTY**: Windows 伪终端支持
- **Job Objects**: 进程作业对象管理

执行流程: 准备 spawn 上下文 → 创建受限令牌 → 设置 ACL → CreatePipe → CreateProcessAsUser → ReadFile 线程 → WaitForSingleObject → 清理

---

## codex-apply-patch — 补丁应用

**路径**: `codex-rs/apply-patch/`

解析和应用自定义补丁格式（非标准 unified diff），使用 `*** Begin Patch` / `*** End Patch` 标记。

### 核心类型

| 类型 | 说明 |
|------|------|
| `Hunk` | `AddFile`, `DeleteFile`, `UpdateFile` |
| `ParseMode` | `Strict`, `Lenient`, `Streaming` |

### 关键功能

- `apply_patch()`: 入口，解析并应用补丁
- `derive_new_contents_from_chunks()`: 基于 diff hunk 计算文件新内容
- `compute_replacements()`: 行匹配 → (start, old_len, new_lines) 替换
- `unified_diff_from_chunks()`: 生成标准 unified diff
- `extract_apply_patch_from_bash()`: 使用 Tree-sitter Bash AST 解析 bash 调用

---

## codex-shell-command — 命令解析与安全检查

**路径**: `codex-rs/shell-command/` | **规模**: ~2500 行分类器

### 核心类型

| 类型 | 说明 |
|------|------|
| `ParsedCommand` | `Read`, `ListFiles`, `Search`, `Unknown` |

### 识别的命令

读取: `cat`, `bat`, `less`, `more`, `head`, `tail`, `awk`, `nl`, `sed -n`
搜索: `rg`/`rga`, `grep`, `git grep`, `ag`/`ack`/`pt`
文件列表: `ls`/`eza`/`exa`, `tree`, `fd`, `find`, `du`, `git ls-files`

### 安全检查

- `is_dangerous_command()`: 识别危险命令（删除、覆盖、提权等）
- `is_safe_command()`: 分类只读安全命令
- Shell 类型检测: `Zsh`, `Bash`, `PowerShell`, `Sh`, `Cmd`

---

## codex-shell-escalation — Unix 权限提升

**路径**: `codex-rs/shell-escalation/`

基于 execve-wrapper 的权限提升：
- Socket 权限提升管理
- 权限配置文件
- 基于会话的授权

---

## codex-process-hardening — 进程加固

**路径**: `codex-rs/process-hardening/`

pre-main 进程加固：
- 禁用 core dumps
- 禁用 ptrace
- 清除 `LD_PRELOAD`、`DYLD_*` 环境变量
- 禁用 macOS malloc stack-logging
