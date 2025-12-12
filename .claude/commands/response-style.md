---
argument-hint: [response-mode]
description: Set AI response verbosity (concise, general, detailed)
---

# Response Style Mode Switcher

## Usage
- `/response-style concise` - Switch to concise mode (brief, direct answers)
- `/response-style general` - Switch to general mode (balanced, standard responses)
- `/response-style detailed` - Switch to detailed mode (comprehensive, thorough explanations)

## Mode Descriptions

### Concise Mode
- Short, direct answers
- Minimal explanations
- Code-only responses when appropriate
- Bullet points for lists
- Maximum efficiency

### General Mode
- Balanced detail level
- Standard explanations
- Appropriate for most tasks
- Clear but not verbose
- Default behavior

### Detailed Mode
- Comprehensive explanations
- Multiple examples
- Thorough reasoning
- Step-by-step breakdowns
- Maximum context

## Implementation Notes
- Mode persists for the conversation
- Can be switched mid-conversation
- Affects response length and detail level
- Does not impact technical accuracy
- Respects existing context boundaries
