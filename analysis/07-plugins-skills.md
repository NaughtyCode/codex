# 插件与技能系统

涵盖 5 个 crate：`codex-plugin`, `codex-core-plugins`, `codex-core-skills`, `codex-skills`, `codex-tools`

---

## codex-plugin — 共享插件标识符

**路径**: `codex-rs/plugin/`

插件系统的共享标识符和元数据类型。

### 核心类型

| 类型 | 说明 |
|------|------|
| `PluginId` | 插件唯一标识符 |
| `PluginLoadOutcome` | 插件加载结果（成功/失败/禁用） |
| `CapabilitySummary` | 插件能力摘要 |
| 命名空间解析 | 连接器/技能命名空间 |

---

## codex-core-plugins — 插件生命周期

**路径**: `codex-rs/core-plugins/`

插件生命周期管理：

### 功能

| 功能 | 说明 |
|------|------|
| **市场管理** | 精选市场、捆绑市场、远程市场 |
| **安装** | 从市场下载和安装插件 |
| **同步** | 自动同步已安装插件 |
| **升级** | 更新到最新版本 |
| **切换** | 启用/禁用插件 |
| **发现** | 从已安装插件发现工具和能力 |

### 插件来源

```
PluginsManager
  ├── 精选市场 (curated marketplace)
  ├── 捆绑插件 (bundled plugins)
  ├── 远程市场 (remote marketplace)
  └── 本地开发 (local dev paths)
```

### 插件注入

插件通过以下方式影响 Agent 行为：
- **指令注入**: 将插件指令注入提示词
- **工具暴露**: 插件定义的工具注册到工具注册表
- **MCP 服务器**: 插件可以提供 MCP 服务器配置
- **技能根目录**: 插件可以提供技能定义

---

## codex-core-skills — 技能引擎

**路径**: `codex-rs/core-skills/`

技能加载、解析和渲染引擎。

### 功能

| 功能 | 说明 |
|------|------|
| **加载** | 从技能根目录加载技能定义 |
| **解析** | 解析技能元数据和引用文件 |
| **渲染** | 渲染技能模板（依赖注入） |
| **环境检测** | 检测所需环境变量 |
| **预算管理** | 渲染预算（token/行限制） |

### 技能解析

```
SkillsManager
  → 从技能根目录发现 SKILL.md 文件
    → 解析 frontmatter (名称, 描述, 触发条件)
      → 解析技能指令
        → 依赖注入解析
          → 环境变量检测
            → 生成 SkillInjections
```

---

## codex-skills — 内置技能

**路径**: `codex-rs/skills/`

编译进二进制的内置技能定义：
- 技能文件以静态资源形式嵌入
- 技能路径解析
- 默认技能集

---

## codex-tools — 共享工具定义

**路径**: `codex-rs/tools/`

Responses API 的共享工具定义。

### 工具类型

| 工具 | 说明 |
|------|------|
| `apply-patch` | 代码补丁应用工具 |
| `exec` | Shell 命令执行工具 |
| `code-mode` | 代码模式执行工具 |
| `MCP tools` | MCP 协议工具代理 |
| `plan` | 计划模式工具 |
| `web_search` | Web 搜索工具 |

### 工具注册表

- `ToolRegistry`: 工具名称 → 工具处理器映射
- `ToolHandler` trait: `kind()`, `handle()`, `is_mutating()`
- 工具路由: 基于配置动态构建可用工具集
- 发现性工具: 从插件和 MCP 服务器动态发现
