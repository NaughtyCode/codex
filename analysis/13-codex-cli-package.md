# npm CLI 封装

`codex-cli/` 是 `@openai/codex` npm 元包的打包和分发层。

---

## 架构概览

```
用户运行: npx @openai/codex <args>
    ↓
bin/codex.js (Node.js ESM 启动器)
  ├── 平台/架构检测 (linux/darwin/win32, x64/arm64)
  ├── 解析平台原生包 (@openai/codex-darwin-arm64 等)
  ├── 生成原生 Rust 二进制 (codex)
  ├── 转发信号 (SIGINT, SIGTERM, SIGHUP)
  └── 镜像退出码
    ↓
原生 Rust 二进制 (codex-rs)
```

---

## 核心组件

### 1. npm 元包 (`package.json`)

**包名**: `@openai/codex`
**构建系统**: pnpm (v10.29.3+)
**Node.js**: ≥ 16

- 声明 `bin/codex.js` 为可执行入口
- `files` 字段包含 `bin` 和 `vendor` 目录
- 无 `dependencies`/`devDependencies` — 依赖可选平台原生子包
- 平台原生包作为 `optionalDependencies` 声明（`@openai/codex-darwin-arm64` 等 6 个）

---

### 2. Node.js 启动器 (`bin/codex.js`)

ESM 脚本，核心功能：

1. **平台检测**
   ```
   process.platform + process.arch → Rust target triple
   例: darwin + arm64 → aarch64-apple-darwin
   ```

2. **二进制解析**
   ```
   优先级:
   1. require.resolve("@openai/codex-{platform}-{arch}")
   2. vendor/{target-triple}/codex/codex (开发回退)
   3. 错误: 提示用户重新安装
   ```

3. **进程管理**
   - 通过 `spawn()` 异步启动，继承 stdio
   - 信号转发: SIGINT, SIGTERM, SIGHUP
   - 退出码镜像: 子进程退出时父进程以相同代码退出

4. **PATH 更新**
   - 注入 `vendor/{target-triple}/path/` 目录（包含 ripgrep）

5. **包管理器检测**
   - `npm_config_user_agent` 检查
   - `npm_execpath` 检查
   - 设置 `CODEX_MANAGED_BY_NPM` 或 `CODEX_MANAGED_BY_BUN`

---

### 3. ripgrep 清单 (`bin/rg`)

**DotSlash 清单**，不是二进制文件。首次使用时按需下载 ripgrep v15.1.0。

支持的 6 平台:
- macos-aarch64, macos-x86_64
- linux-aarch64, linux-x86_64
- windows-x86_64, windows-aarch64

---

## Docker 沙箱

### Dockerfile

- 基于 `node:24-slim`
- 安装系统工具: git, gh, jq, ripgrep, fzf, iptables/ipset, zsh 等
- 全局安装 `codex.tgz` npm 包
- 设置 `CODEX_UNSAFE_ALLOW_NO_SANDBOX=1`
- 以非 root `node` 用户运行

### 防火墙 (`init_firewall.sh`)

沙箱容器内的 iptables 出口流量限制：
1. 刷新所有现有 iptables 规则和 ipsets
2. 允许 DNS (UDP 53) 和 localhost
3. 从 `/etc/codex/allowed_domains.txt` 创建 `allowed-domains` ipset
4. 默认策略: INPUT/FORWARD/OUTPUT → DROP
5. 出站仅允许已解析域 IP
6. TCP/UDP 拒绝并发送 reset/unreachable

### 容器运行器 (`run_in_container.sh`)

Docker 中运行 codex，带防火墙：
- 挂载工作目录
- 传递 `OPENAI_API_KEY`
- 添加 `NET_ADMIN`/`NET_RAW` capabilities
- 运行 `codex --full-auto <command>`
- 完成后自动清理容器

### 容器构建器 (`build_container.sh`)

编排: pnpm install → pnpm pack → docker build -t codex

---

## 构建脚本

### `install_native_deps.py`
从 GitHub Actions 工作流下载原生 Rust 二进制到 `vendor/`：
- 6 平台 × 2-3 组件（`codex`, `codex-windows-sandbox-setup`, `codex-command-runner`）
- 通过 DotSlash 清单下载 ripgrep
- `ThreadPoolExecutor` 并发下载

### `build_npm_package.py`
构建和打包 npm tarball：
- `codex`（元包）: 复制启动脚本和清单，生成含 `optionalDependencies` 的 `package.json`
- `codex-{platform}`（6 个平台包）: 含 `os`/`cpu` 限制，嵌入原生二进制
- `codex-responses-api-proxy`: 代理包
- `codex-sdk`: TypeScript SDK 包
- 运行 `npm pack` 生成 `.tgz`

### `stage_npm_packages.py`（仓库根级别）
从 GitHub Actions 工作流运行中准备 npm 包，支持包扩展、版本管理和平台特定命名。

---

## 支持平台

| 目标三元组 | npm 包名 |
|-----------|---------|
| `aarch64-apple-darwin` | `@openai/codex-darwin-arm64` |
| `x86_64-apple-darwin` | `@openai/codex-darwin-x64` |
| `aarch64-unknown-linux-gnu` | `@openai/codex-linux-arm64` |
| `x86_64-unknown-linux-gnu` | `@openai/codex-linux-x64` |
| `x86_64-pc-windows-msvc` | `@openai/codex-win32-x64` |
| `aarch64-pc-windows-msvc` | `@openai/codex-win32-arm64` |
