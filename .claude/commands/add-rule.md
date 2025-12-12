---
description: Create a rule to prevent recurring AI behavior issues
argument-hint: <problem description>
---

User reported a recurring AI behavior issue: $ARGUMENTS

1. Check if already covered by Claude Code defaults (e.g., avoid over-engineering) - if so, inform user
2. Check existing `.claude/rules/` for duplicates
3. If new rule needed, create a minimal `.claude/rules/<name>.md`:
   - Use kebab-case filename
   - Add `paths` frontmatter only if path-specific
   - No redundant explanations
4. Confirm what was created and why
