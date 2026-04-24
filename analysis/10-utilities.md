# 工具库

涵盖 codex-rs/utils/ 下的 23 个实用工具 crate。

---

## 路径与文件系统

### codex-utils-absolute-path
`AbsolutePathBuf` 类型 — 保证绝对且标准化的路径，支持 serde 序列化和主目录展开。在初始化时验证路径为绝对路径。

### codex-utils-home-dir
解析 Codex 主目录（`~/.codex` 或 `$CODEX_HOME` 环境变量）。

### codex-utils-path
路径标准化、符号链接解析、WSL 路径处理和原子文件写入。处理 Windows/Linux/macOS 的路径差异。

---

## 进程与终端

### codex-utils-pty
带 PTY 和 pipe 后端的进程生成：
- PTY 分配和终端大小管理
- 进程组管理
- 跨平台进程句柄
- 用于 shell 命令执行

### codex-utils-cargo-bin
在 Bazel/cargo 构建环境中定位同级二进制文件。使用 `CARGO_BIN_EXE_*` 环境变量和 runfiles 进行定位。

### codex-utils-readiness
基于 token 的异步就绪标志，用于协调组件启动。一次性 token，支持等待和检查就绪状态。

---

## 文本处理

### codex-utils-string
字符串工具：
- Token 计数（用于上下文窗口管理）
- 字节/字符截断
- Metric 标签清理（OpenTelemetry 兼容性）

### codex-utils-stream-parser
流式文本解析器：
- 助手文本流解析
- 引用和内联标签解析
- 计划模式段检测
- UTF-8 流边界处理

### codex-utils-fuzzy-match
大小写不敏感子序列模糊匹配器，返回匹配字符索引和分数。用于文件搜索和 Tab 补全。

### codex-utils-template
最小化 `{{ placeholder }}` 字符串模板引擎。用于提示词和文本资源的简单模板替换。

### codex-utils-elapsed
人类可读的时长格式化（如 "3s", "1m 05s"）。

---

## 数据处理

### codex-utils-image
图像处理和编码：
- 图像缩放（保持宽高比）
- JPEG/PNG/WebP 编码
- Base64 编码
- LRU 缓存用于编码图像

### codex-utils-json-to-toml
将 `serde_json::Value` 转换为语义等效的 `toml::Value`。

### codex-utils-cache
最小化 LRU 缓存，带 Tokio 互斥保护。包含 SHA-1 摘要辅助函数。

---

## CLI 与配置

### codex-utils-cli
共享 CLI 参数类型：
- 审批模式（`ApprovalMode`）
- 沙箱模式（`SandboxMode`）
- 配置覆盖（`-c key=value`）
- 共享 CLI 选项

### codex-utils-approval-presets
内置审批/沙箱预设对（Read Only, Auto 等）。将审批模式与沙箱策略配对。

### codex-utils-sandbox-summary
沙箱策略和配置摘要生成，用于向用户显示当前沙箱设置。

### codex-utils-output-truncation
使用 `TruncationPolicy` 截断工具和执行输出。按字节/token 预算限制输出大小。

---

## 平台与网络

### codex-utils-rustls-provider
确保进程范围内的 rustls crypto 提供者（ring）安装一次。全局初始化保护。

### codex-utils-sleep-inhibitor
跨平台空闲睡眠预防：
- macOS: IOKit `IOPMAssertionCreateWithName`
- Linux: `systemd-inhibit`
- Windows: `PowerRequest` API

### codex-utils-plugins
插件路径解析、mention 语法 sigils 和 MCP 连接器辅助函数。

### codex-utils-oss
TUI 和 exec 之间共享的 OSS 提供商工具：
- 默认模型解析
- LM Studio / Ollama 就绪检查
- 模型拉取进度共享工具
