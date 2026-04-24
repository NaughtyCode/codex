# 执行策略引擎

涵盖 2 个 crate：`codex-execpolicy`, `codex-execpolicy-legacy`

---

## codex-execpolicy — Starlark 执行策略引擎

**路径**: `codex-rs/execpolicy/`

基于 **Starlark** 脚本语言的新一代执行策略引擎，评估命令审批规则。

### 架构

```
ExecPolicyManager
  → 从每个配置层加载 .rules 文件
    → Starlark 引擎评估
      → 审批决策: Allow | Prompt | Forbidden
```

### Starlark 规则格式

规则定义为 Starlark 代码片段，支持：
- **前缀匹配**: 基于命令前缀的审批决策
- **条件逻辑**: 参数检查、路径匹配、环境判断
- **运行时修正**: 允许列表动态添加

### 决策输出

| 决策 | 说明 |
|------|------|
| `Allow` | 命令可自由运行 |
| `Prompt` | 需要用户确认 |
| `Forbidden` | 命令被阻止 |

### 集成点

- 由 `codex-core` 的 `ExecPolicyManager` 加载和调用
- 规则从每个配置层的 `rules/` 子目录加载
- 支持配置层叠加（用户规则 + 项目规则 + 组织策略）
- 当无显式规则匹配时，应用启发式安全检查

---

## codex-execpolicy-legacy — 旧版执行策略

**路径**: `codex-rs/execpolicy-legacy/`

Starlark 之前的旧版执行策略引擎。

### 规则格式

使用较旧的规则格式（TOML 或自定义格式），包含：
- 命令前缀匹配
- 审批级别（allow/prompt/forbidden）
- 基本条件检查

### 迁移状态

- 标记为 "Legacy exec policy engine for validating proposed exec calls"
- 逐步迁移到 Starlark 引擎
- 两套系统在过渡期内共存

---

## 策略评估流程

```
Shell 命令执行请求
  → ExecPolicyManager.check()
    → 加载策略规则（各配置层 rules/ 目录）
      → Starlark 引擎评估（或旧版引擎）
        → 命令前缀匹配
          → 检查条件（参数、路径等）
            → 返回决策
              ├── Allow → 执行命令
              ├── Prompt → 发送审批请求到用户/Guardian
              └── Forbidden → 阻止执行，返回错误
    → 如无规则匹配 → 启发式安全检查
      → 检查危险模式（rm -rf, sudo, chmod 777 等）
```

### 配置文件位置

| 层级 | 路径 |
|------|------|
| 云需求 | 后端下发 |
| 管理员/MDM | macOS 管理设备策略 |
| 系统 | `/etc/codex/rules/` |
| 用户 | `$CODEX_HOME/rules/` |
| 项目 | `.codex/rules/` |
| 仓库 | git root `.codex/rules/` |
