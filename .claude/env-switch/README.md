# Claude 环境切换工具

## 快速切换 Claude API 环境

可用的环境：
- **env_kimi** - Kimi API
- **env_minimax** - MiniMax API
- **env_glm** - GLM API

## 使用方法

### 方法 1：VSCode Tasks（推荐）
1. `Ctrl+Shift+P` → "Tasks: Run Task"
2. 选择环境任务：
   - `Switch to Kimi Environment`
   - `Switch to MiniMax Environment`
   - `Switch to GLM Environment`
   - `Show Available Environments`

### 方法 2：命令行
```bash
node .claude/env-switch/index.js --env env_kimi
node .claude/env-switch/index.js --env env_minimax
node .claude/env-switch/index.js --env env_glm
```

## 工作原理

- **源文件**: `.claude\.cache\env.local.json`
- **目标文件**: `.claude\settings.local.json`
- 切换后立即生效，无需重启 VSCode
