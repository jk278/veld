---
description: Remove AI code slop from the current branch
---

Review diff against main and remove AI-generated code slop:

1. Run `git diff main...HEAD` to see all changes in the current branch
2. Review the diff for AI-generated slop:
   - Extra comments that a human wouldn't add or inconsistent with the file's style
   - Unnecessary defensive checks or try/catch blocks in trusted codepaths
   - `as any` type casts to bypass type checking
   - Any code style inconsistent with the rest of the file
3. For each identified issue:
   - Read the affected file
   - Remove or fix the slop
   - Ensure the fix maintains correct functionality
4. After making all changes, provide a 1-3 sentence summary of what was changed

Focus on quality over quantity - only remove actual slop, not legitimate code.
