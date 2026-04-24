# 构建与安装脚本

`scripts/` 目录包含构建、CI、安装和开发辅助脚本。

---

## 代码质量

### `asciicheck.py`
**非 ASCII 字符检查器**
- 扫描文件中 ASCII 可打印范围（0x20-0x7E）外的 Unicode 字符
- `--fix` 模式：替换常见排版字符（不可分断空格、破折号、引号、省略号 → ASCII 等价物）
- 防止不可见字符（如 U+00A0）破坏正则表达式或 GitHub Markdown 渲染

### `check_blob_size.py`
**CI 文件大小策略执行器**
- 比较两个 git 版本间变更的文件，检查每个文件大小是否超过可配置的预算（默认 500 KiB）
- 支持白名单文件和二进制文件检测
- 写入 GitHub Actions Step Summary

### `check-module-bazel-lock.sh`
**Bazel Lock 一致性检查**
- 运行 `bazel mod deps --lockfile_mode=error`
- 如果 `MODULE.bazel.lock` 过期则失败，提示用户运行 `just bazel-lock-update`

---

## Bazel 目标列表

### `list-bazel-clippy-targets.sh`
输出应使用 Clippy（Rust linter）构建的 Bazel 目标列表：
- `//codex-rs/...` 除去 `//codex-rs/v8-poc:all`
- 解析 `manual` `rust_test` 目标以确保 `#[cfg(test)]` 代码也被 lint

### `list-bazel-release-targets.sh`
输出 Release 模式的 Bazel 目标：
- `//codex-rs/...` 除去 `//codex-rs/v8-poc:all`

---

## 开发辅助

### `debug-codex.sh`
**VS Code 调试便捷脚本**
- 设置为 VS Code 设置中的 `chatgpt.cliExecutable`
- 切换到 `codex-rs/` 目录并运行 `cargo run --quiet --bin codex`
- 无需手动重新编译

### `mock_responses_websocket_server.py`
**本地测试 Mock WebSocket 服务器**
- 模拟 OpenAI Responses API over WebSocket
- 两请求流程: Request 1 触发 function call，Request 2 响应助手消息
- 打印 config.toml 配置片段

### `run_tui_with_exec_server.sh`
**TUI + Exec Server 组合启动**
- 后台启动 `codex exec-server`
- 等待其输出 WebSocket 监听 URL
- 设置 `CODEX_EXEC_SERVER_URL` 启动 `codex-tui`
- 退出时清理 server 进程

---

## 远程执行

### `start-codex-exec.sh`
**远程 Exec Server 同步和启动**
- 通过 rsync/SSH 将本地仓库同步到远程主机
- 在远程机器上执行 `cargo build` 构建 `codex exec-server`
- SSH 隧道将远程 exec-server 端口映射到本地端口
- 退出时清理远程服务器和隧道

### `test-remote-env.sh`
**远程测试 Docker 环境**
- 必须 source（不直接执行）
- 构建 Ubuntu 24.04 Docker 容器（含 bubblewrap 兼容设置）
- 安装依赖（Python 3, zsh）、复制 codex 二进制
- 启动容器内 exec-server
- 导出 `CODEX_TEST_REMOTE_ENV` 和 `CODEX_TEST_REMOTE_EXEC_SERVER_URL`

---

## 安装器

### `install.ps1` (Windows)
**Windows Codex CLI 安装器**
- 从 GitHub Releases 下载发布版（可指定版本）
- SHA-256 校验
- 安装到 `%USERPROFILE%\.codex\packages\standalone\releases\`
- 使用 NTFS junctions（C# P/Invoke）管理 `current` 符号链接
- PATH 管理（注册表操作）
- 冲突检测：npm/bun 管理的安装
- 旧版布局迁移
- 文件级安装锁

### `install.sh` (macOS/Linux)
**POSIX Codex CLI 安装器**
- 支持 `--release VERSION` 指定版本
- OS + 架构自动检测
- 从 GitHub API 解析发布（或版本标签如 `rust-v0.1.0`）
- 下载 npm 风格 tarball，SHA-256 校验
- 安装到 `~/.codex/packages/standalone/` 带 `current` 符号链接
- 在 `~/.local/bin`（或 `$CODEX_INSTALL_DIR`）创建 `codex` 符号链接
- Shell profile PATH 管理（标记分隔的块）
- 目录安装锁 + 过期检测
- 冲突检测：npm/brew 安装
- 可选的安装后启动

---

## 发布

### `stage_npm_packages.py`
**npm 发布包构建**
- 从 GitHub Actions 构建工作流下载和准备原生组件
- 解析包扩展：一个名称 → 多个平台包
- 平台特定命名：`codex-npm-win32-x64-0.1.0.tgz`
- 支持重用特定工作流运行的产物

### `readme_toc.py`
**README 目录生成**
- 查找 `<!-- Begin ToC -->` / `<!-- End ToC -->` 标记
- 提取 `##` 到 `######` 标题（跳过代码块）
- 生成嵌套 anchor 链接列表
- `--fix` 重写文件；无 `--fix` 显示 unified diff
