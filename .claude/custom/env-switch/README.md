# Claude 环境切换工具

快速切换 Claude API 环境

可用的环境：
- **env_kimi** - Kimi API
- **env_minimax** - MiniMax API
- **env_glm** - GLM API

## 配置文件

⚠️ **使用前需要创建配置文件**：
参考 `.env.example` 创建 `.claude\custom\env-switch\env.local.json` 文件，格式如下：

```json
{
  "env_kimi": {
    "ANTHROPIC_BASE_URL": "...",
    "ANTHROPIC_AUTH_TOKEN": "...",
    ...
  },
  "env_xxx": { ... },
  ...
}
```

## 使用方法

### 方法 1：VSCode Tasks（推荐）
1. `Ctrl+Shift+P` → "Tasks: Run Task"
2. 选择 `Switch Env for Claude` 任务
3. 在弹出的交互式列表中选择要切换的环境

### 方法 2：命令行
```bash
# 以 Kimi 环境为例
node .claude/custom/env-switch/index.js --env env_kimi
```

## 工作原理

- **源文件**: `.claude\custom\env-switch\env.local.json`
- **目标文件**: `.claude\settings.local.json`
- 切换后需要在命令面板Refresh Claude Code Cli Agent Sessions，无需重启 VSCode
