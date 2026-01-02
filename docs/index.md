# Veld 文档

## 📚 文档索引

### ⚡ 快速参考

- **[Dioxus 0.7 Desktop API 快速参考](./DIoxus_Desktop_API_QuickRef.md)**
  - 全局快捷键：`use_global_shortcut`
  - 事件循环：`use_wry_event_handler`
  - 系统托盘：`use_tray_icon_event_handler`
  - 窗口管理：`use_wry_window`, `DesktopContext`

  > ⚠️ **必读！** 避免重复造轮子，先查看此文档！

---

### 🏗️ 架构设计

- **[Agent 链式调用架构](./ARCHITECTURE.md)** ⭐
  - 设计哲学：步骤流、ID-based 更新、链式调用
  - 数据流：Agent → Step → UI → History
  - Step 类型：Tool/Info/Answer
  - 实现细节：hooks.rs, agent.rs, message_list.rs

  > 🎯 **核心！** 理解此文档是修改 Agent 代码的前提！

---

### 📖 项目文档

- **[项目计划与进度](../.claude/CLAUDE.md)**
  - 项目概述
  - 技术栈
  - 实施阶段
  - 里程碑

---

## 🚀 快速开始

1. **阅读架构文档**：先看 `ARCHITECTURE.md` 理解 Agent 设计
2. **查看 API 参考**：阅读 `DIoxus_Desktop_API_QuickRef.md` 了解内置能力
3. **查看示例代码**：在 `src/` 目录下查找相关示例

---

## 💡 开发提示

- ✅ **优先使用内置 API**：dioxus-desktop 已经封装了大部分原生功能
- ✅ **事件循环自动管理**：所有 Hook 都会自动在正确的时机注册/注销
- ✅ **理解步骤流**：Agent 输出的是 Step 流，不是独立的消息
- ✅ **ID 更新机制**：相同 ID 的步骤会更新，而非追加


