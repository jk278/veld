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

### 📖 项目文档

- **[项目计划与进度](../.claude/CLAUDE.md)**
  - 项目概述
  - 技术栈
  - 实施阶段
  - 里程碑

---

## 🚀 快速开始

1. **阅读 API 参考**：先看 `DIoxus_Desktop_API_QuickRef.md` 了解内置能力
2. **查看示例代码**：在 `src/` 目录下查找相关示例
3. **参考项目计划**：在 `.claude/CLAUDE.md` 中查看整体规划

---

## 💡 开发提示

- ✅ **优先使用内置 API**：dioxus-desktop 已经封装了大部分原生功能
- ✅ **事件循环自动管理**：所有 Hook 都会自动在正确的时机注册/注销
- ✅ **参考示例项目**：dioxus 仓库中有完整的示例代码

