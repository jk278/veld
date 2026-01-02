# Veld Agent Architecture

## 设计哲学：工具即结果，结果即工具

### 核心洞察

传统 AI Agent 架构的问题：
```
❌ 错误：工具调用和答案被人为分离
工具阶段 → [工具1, 工具2, 工具3] → 答案阶段 → 生成文本
```

真正的优雅架构：
```
✅ 正确：工具调用是答案的一部分
AI 输出流 → [文本 + 工具调用 + 文本 + 工具调用 + ...] → 完成
```

### 核心原则

1. **工具即结果** - 工具调用是 AI 回答的一部分，不是特殊阶段
2. **结果即工具** - AI 的输出可能包含工具调用，需要实时检测和执行
3. **流式检测** - 边流边检测工具，边执行边继续流
4. **自我迭代** - AI 根据工具结果决定下一步，无需人为控制
5. **统一处理** - 所有输出都是 Answer，可能包含工具调用

### 关键决策

```
传统架构（已废弃）：
Agent → 检测类型 → 工具/答案双路径 → 复杂同步

新架构（工具即结果）：
Agent → 流式 Answer → 实时检测工具 → 执行并继续流
```

**为什么删除"检测阶段"？**
- 破坏流的连续性
- 增加认知负担（AI 需要决定"工具"还是"答案"）
- 不符合自然对话流程

**为什么允许工具嵌入 Answer？**
- 符合自然对话（说话中决定调用工具）
- 支持自我迭代（看到结果后继续思考）
- 无需分离"工具阶段"和"答案阶段"

---

## 数据流

```
用户输入
  ↓
Agent 开始流式输出
  ↓
实时检测：内容中是否包含工具调用？
  ├─ 是 → 执行工具 → 将结果追加到上下文 → 继续流（不中断！）
  └─ 否 → 继续流式输出
  ↓
直到 AI 发送完成信号
  ↓
保存到历史
```

### 流的连续性

```
AI 输出：| 文本部分 | 工具调用JSON | 更多文本 | 工具调用JSON | 最终答案 |
          ↓           ↓           ↓          ↓           ↓
          流式展示    检测并执行    流式展示    检测并执行    流式完成
```

---

## 执行流程

### 核心循环

```rust
// agent.rs: chat_with_tools()

loop {
    // 1. 接收流式 chunk
    let chunk = rx.recv().await?;

    // 2. 累积到 buffer
    accumulated.push_str(&chunk);
    answer_buffer.push_str(&chunk);

    // 3. 检测是否有工具调用
    while let Some(tool_call) = extract_tool_call(&accumulated) {
        // 执行工具
        let result = execute_tool_call(&tool_call, &mut clients)?;

        // 追加到上下文（让 AI 知道结果）
        current_messages.push(...tool result...);

        // 继续流（新的 client.chat() 调用）
        rx = client.chat(current_messages.clone()).await?;
        accumulated.clear();

        break; // 重新进入接收循环
    }

    // 4. 没有工具调用？流式展示
    if !answer_buffer.is_empty() {
        tx.send(Step::answer(&answer_buffer, false));
        answer_buffer.clear();
    }
}
```

### 多工具调用示例

```
AI: "让我帮您查询文档..."
    ↓ (流式输出)
AI: "...我需要先调用 resolve-library-id 查找库 ID"
    ↓ (检测到工具调用 JSON)
执行: resolve-library-id("dioxus")
    ↓ (结果返回，追加到上下文)
AI: "好的，找到了 /dioxuslabs/docsite，现在查询文档..."
    ↓ (继续流式输出)
AI: "...{\"tool_call\": {\"name\": \"query-docs\", ...}}"
    ↓ (检测到工具调用 JSON)
执行: query-docs("/dioxuslabs/docsite", "快速开始指南")
    ↓ (结果返回，追加到上下文)
AI: "根据文档，Dioxus 是一个跨平台 UI 框架..." (最终答案)
```

---

## Step 类型

```rust
pub enum Step {
    /// 工具调用（带状态）
    Tool {
        id: String,              // 用于更新 (tool-0, tool-1, ...)
        name: String,
        args: Value,
        result: Option<String>,
        status: ToolStatus,      // 状态可视化
        timestamp: u64,
    },

    /// 信息提示
    Info {
        id: String,              // 用于更新
        text: String,
        timestamp: u64,
    },

    /// AI 输出（可能包含工具调用！）
    Answer {
        content: String,         // 流式累积
        done: bool,              // 完成标记
        timestamp: u64,
    },
}
```

### Answer 的双重性

```
Answer.content 可能是：
1. 纯文本："这是一个答案"
2. 纯工具调用：`{"tool_call": {...}}`
3. 混合内容："让我帮您...{"tool_call": {...}}...好的，结果是..."
```

检测逻辑会自动识别并处理第 2、3 种情况。

---

## 工具调用检测

### 嵌入式检测

```rust
fn extract_tool_call(text: &str) -> Option<ToolCall> {
    // 1. 尝试直接解析整个文本
    if let Ok(v) = serde_json::from_str::<Value>(text) {
        if let Some(tc) = v.get("tool_call") {
            return serde_json::from_value(tc.clone()).ok();
        }
    }

    // 2. 检测嵌入的 JSON
    if text.contains("\"tool_call\"") {
        // 手动提取 {...} 内容
        // 支持嵌入在文本中的工具调用
    }

    None
}
```

### 检测时机

```
accumulated content: "让我查询..."
    ↓ (无工具调用，继续流)
accumulated content: "让我查询...{\"tool_call\": {\"name\": \"...\"}}"
    ↓ (检测到工具调用，执行它)
accumulated content: "" (清空，准备接收新内容)
    ↓ (AI 继续输出)
```

---

## ID-based 更新机制

### 工具步骤

```rust
// 工具调用发送 3 个状态变化
Step::tool_pending("tool-0", name, args)   // 显示 ⏳
Step::tool_running("tool-0", name, args)   // 显示 🔄
Step::tool_success("tool-0", name, args, result)  // 显示 ✓

// hooks.rs: 相同 ID 会被更新
if let Some(pos) = msgs.iter().position(|m| m.id == "tool-0") {
    msgs[pos].content = new_content;  // 替换而非追加
}
```

### Answer 累积

```rust
// Answer 始终累积到最后一个 answer-* 消息
Step::answer("Hello", false)  // 创建 answer-{timestamp}
Step::answer(" World", false)  // 累积 → "Hello World"
Step::answer("", true)         // 完成标记
```

---

## 链式调用示例

### 场景：查询文档并总结

```
用户: "查询 Dioxus 文档并总结快速开始"

Iteration 0:
  Answer chunk: "好的，让我先查找库 ID..."

  检测到工具: resolve-library-id("dioxus")
  执行: 返回 /dioxuslabs/docsite
  追加到上下文

Iteration 1:
  Answer chunk: "找到库了，现在查询快速开始..."

  检测到工具: query-docs("/dioxuslabs/docsite", "快速开始")
  执行: 返回文档内容
  追加到上下文

Iteration 2:
  Answer chunk: "根据文档，Dioxus 是..."
  Answer chunk: "...一个跨平台框架..."
  Answer chunk: "" (done)
```

### 关键特性

1. **连续流** - 工具调用不中断流，AI 继续输出
2. **自我迭代** - AI 看到结果后自动决定下一步
3. **无需显式控制** - AI 不是在"工具模式"或"答案模式"

---

## 历史同步

### 保存时机

```rust
// execute_agent() 完成后统一保存
ops.save_to_history();
```

### 保存内容

所有 `role = "assistant"` 的消息：
- Tool 步骤（JSON 格式）
- Info 步骤（纯文本）
- Answer 步骤（Markdown）

### 为什么不实时保存？

- **性能** - 避免每次 Step 都写磁盘
- **原子性** - 完成后一次性保存，数据一致
- **简洁性** - 无需复杂的增量同步逻辑

---

## 错误处理

### Agent 错误

```rust
if let Err(e) = chat_with_tools(messages, step_tx, abort_rx).await {
    eprintln!("[HOOKS] Agent error: {:?}", e);
}
```

错误不中断执行，只打印日志。UI 保持最后状态。

### 工具执行失败

```rust
Step::tool_success(id, name, args, result)  // result 可能包含错误信息
```

UI 显示 `✗ 失败`，但 AI 会继续执行（可能生成错误说明或尝试其他工具）。

---

## 性能考虑

### 优势

1. **无循环中断** - 工具调用不阻塞流
2. **即时反馈** - 用户实时看到工具执行
3. **无需预检测** - 不在工具阶段前检测

### 优化空间

1. **批量检测** - 一次检测多个工具调用
2. **并行执行** - 如果 AI 同时调用多个工具
3. **缓存优化** - 缓存常用工具的结果

---

## 与传统架构对比

### 传统架构

```
问题：
1. 工具调用和答案分离
2. AI 需要明确决定"工具"或"答案"
3. 复杂的状态机和阶段管理
4. 难以支持自我迭代
```

### 工具即结果架构

```
优势：
1. 工具调用嵌入答案
2. AI 自然输出，无需特殊格式
3. 简单的流式检测
4. 自我迭代原生支持
```

---

## 扩展性

### 添加新的 Step 类型

```rust
pub enum Step {
    Tool { ... },
    Info { ... },
    Answer { content, done, timestamp },

    // 新增：文件操作
    File {
        id: String,
        path: String,
        content: String,
        status: FileStatus,
        timestamp: u64,
    },
}
```

### 支持多 Agent

```rust
pub enum Step {
    Tool { ... },
    Info { ... },
    Answer { ... },

    // 新增：子 Agent 调用
    SubAgent {
        id: String,
        agent_name: String,
        input: String,
        output: Option<String>,
        status: AgentStatus,
    },
}
```

---

## 总结

### 架构优势

1. **工具即结果** - 工具调用是答案的一部分
2. **结果即工具** - AI 输出可能包含工具调用
3. **流式检测** - 边流边检测工具，边执行边继续流
4. **自我迭代** - AI 根据结果自动决定下一步
5. **扁平架构** - 无复杂状态机和阶段管理

### 设计原则

- **自然对话** - AI 像对话一样输出，无需特殊格式
- **实时反馈** - 用户实时看到工具执行
- **简单优先** - 能直接检测就不要预解析
- **流式优先** - 保持流的连续性

### 未来改进

1. **多工具并行** - 检测多个工具调用，并行执行
2. **流式工具结果** - 工具结果也可以流式返回
3. **上下文优化** - 智能管理上下文窗口
4. **增量保存** - 历史记录的增量保存
