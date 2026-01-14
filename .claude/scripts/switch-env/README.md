# Claude Code Environment Switcher

Switch between different LLM API providers (Kimi/MiniMax/GLM).

## Usage

Via slash commands:
```bash
/env/kimi      # Switch to Kimi API
/env/minimax   # Switch to MiniMax API
/env/glm       # Switch to GLM API
```

## Configuration

Create `.claude/scripts/switch-env/env.local.json`:

```json
{
  "env_kimi": {
    "ANTHROPIC_BASE_URL": "",
    "ANTHROPIC_AUTH_TOKEN": "",
    "ANTHROPIC_MODEL": "",
    "ANTHROPIC_SMALL_FAST_MODEL": ""
  },
  "env_minimax": { ... },
  "env_glm": { ... }
}
```

## Post-Switch

After switching, refresh Claude Code CLI Agent Sessions (no VSCode restart needed).
