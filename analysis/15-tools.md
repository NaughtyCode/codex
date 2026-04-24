# 开发者工具

`tools/` 目录包含一个开发者工具。

---

## argument-comment-lint

**路径**: `tools/argument-comment-lint/`
**类型**: Rust Dylint 库 + 自定义 `rustc_driver` 二进制 + Bazel aspect + Python 封装

### 用途

自定义 Rust lint，强制在 Rust 代码中使用 `/*param_name*/` 参数注释。作为 Dylint 库加载到 `rustc`，并集成到项目的 Bazel 构建系统中作为构建 aspect。

---

### Lint 规则

#### 1. `ARGUMENT_COMMENT_MISMATCH` (默认 warn)

当函数调用中发现 `/*param*/` 注释时，验证其与解析的调用参数名匹配。不匹配的注释被标记为**主动误导**。

```rust
// 正确: 注释与参数名匹配
foo.do_something(/*base_url*/ "https://...");

// 错误: 注释 "api_base" 应为 "base_url" → 触发 ARGUMENT_COMMENT_MISMATCH
foo.do_something(/*api_base*/ "https://...");
```

#### 2. `UNCOMMENTED_ANONYMOUS_LITERAL_ARGUMENT` (默认 allow)

标记缺少前置 `/*param*/` 注释的裸字面量参数：
- 被标记: `None`, booleans, 数字字面量
- 豁免: 字符串和 char 字面量（被认为是自描述的）
- 范围: 仅工作区内部函数（名称以 `codex_` 开头或测试支持 crate）

```rust
// 被标记:
foo.do_something(None);     // 哪个参数是 None？
foo.process_data(3);        // 3 代表什么？
bar.enable_debug(true);     // true 的意图不清

// 不标记:
foo.do_something(/*count*/ 3, /*sort*/ true);
foo.do_something("hello");  // 字符串是自描述的
```

---

### 架构

#### Rust 组件

| 文件 | 说明 |
|------|------|
| `src/lib.rs` | Dylint lint 库：定义两个 lint，实现 `LateLintPass` |
| `src/comment_parser.rs` | 独立解析器：从 Rust 源文本中提取 `/*identifier*/` |
| `driver.rs` | 自定义 `rustc_driver` 二进制：注册 lint 并作为 Bazel aspect 运行 |
| `src/bin/argument-comment-lint.rs` | 在 `ui/` 测试夹具上运行 lint 的辅助二进制 |

#### 构建规则

| 文件 | 说明 |
|------|------|
| `BUILD.bazel` | 两个 Bazel 目标：`argument-comment-lint-lib`（rust_library）+ `argument-comment-lint-driver`（rust_binary） |
| `lint_aspect.bzl` | Bazel aspect `rust_argument_comment_lint_aspect`：在构建中注入驱动，设置 `-Dargument-comment-mismatch` 和 `-Duncommented-anonymous-literal-argument` 严格标志 |
| `list-bazel-targets.sh` | 输出所有应 lint 的 Bazel 目标（含 `#[cfg(test)]` 代码） |

#### 运行封装

| 文件 | 说明 |
|------|------|
| `run.py` | 源代码构建封装：`cargo dylint --path <lint-dir>` |
| `run-prebuilt-linter.py` | 预编译二进制封装：使用 DotSlash 获取发布包 |
| `wrapper_common.py` | 共享逻辑：参数解析、环境设置、rustup shim 管理 |
| `argument-comment-lint` | DotSlash 清单 |

---

### 集成点

| 集成 | 调用方式 |
|------|----------|
| **Bazel** | `bazel build --config=argument-comment-lint //codex-rs/...` |
| **Just** | `just argument-comment-lint`（工作区范围）或 `just argument-comment-lint -p codex-core`（单 crate） |
| **CI** | GitHub Actions workflow 中的 `list-bazel-targets.sh` |
| **GitHub Releases** | 预编译包（macOS arm64, Linux arm64/x64, Windows x64） |

---

### 环境配置

- 工具链: `nightly-2025-09-18`（通过 `rust-toolchain` 文件固定）
- 组件: `llvm-tools-preview`, `rustc-dev`, `rust-src`
- 链接器: `dylint-link`（通过 `.cargo/config.toml`）
