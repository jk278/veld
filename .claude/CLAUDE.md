# Veld - AI Toolkit for Developers
## Dioxus 0.7 练手项目计划

### 🎯 项目概述
构建一个跨平台系统托盘工具，通过键盘快捷键快速访问 AI 助手功能。支持预配置提示词、上下文菜单和实时 AI 交互，提升开发效率和工作流。

### 📁 项目结构
```
veld/
├── Cargo.toml              # 项目配置
├── src/
│   ├── main.rs            # 应用入口 + 系统托盘初始化
│   ├── app.rs             # 浮动输入窗口主组件
│   ├── tray.rs            # 系统托盘菜单与快捷键处理
│   ├── components/        # UI 组件
│   │   ├── mod.rs
│   │   ├── floating_input.rs # 浮动输入窗口
│   │   ├── result_viewer.rs  # AI 结果显示
│   │   └── tool_selector.rs  # 工具选择器
│   ├── services/          # AI 与工具服务
│   │   ├── mod.rs
│   │   ├── ai_client.rs   # AI API 客户端 (OpenAI/Anthropic/Local)
│   │   ├── shortcuts.rs   # 全局快捷键注册
│   │   ├── tools/         # 预配置工具
│   │   │   ├── mod.rs
│   │   │   ├── summarize.rs
│   │   │   ├── translate.rs
│   │   │   ├── code_gen.rs
│   │   │   ├── explain.rs
│   │   │   └── refactor.rs
│   │   └── clipboard.rs   # 剪贴板操作
│   ├── config/            # 配置管理
│   │   ├── mod.rs
│   │   ├── settings.rs    # 用户设置
│   │   └── shortcuts.rs   # 快捷键配置
│   └── utils/             # 工具函数
│       ├── mod.rs
│       ├── window_manager.rs # 窗口管理
│       └── theme.rs       # 主题系统
├── assets/
│   ├── icons/            # 应用图标 (托盘、通知)
│   └── styles.css        # 全局样式
├── config/
│   └── default_config.json # 默认配置
└── README.md
```

### 🛠️ 技术栈

#### 核心依赖
```toml
[package]
name = "veld"
version = "0.1.0"
edition = "2021"

[dependencies]
# Dioxus
dioxus = "0.7.0"
dioxus-desktop = "0.7.0"
dioxus-signals = "0.7.0"
dioxus-hooks = "0.7.0"

# Async runtime
tokio = { version = "1.40", features = ["full"] }
futures = "0.3"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# System Tray & Global Shortcuts
system_tray = "0.9"
winit = "0.30"  # For global shortcuts
tao = "0.30"    # For window management

# Clipboard
clipboard = "0.5"

# HTTP client (for AI APIs)
reqwest = { version = "0.12", features = ["json", "stream"] }
openai-api = "0.8"  # Or anthropic, or generic HTTP

# Notification
notify-rust = "0.5"

# File watching
notify = "6.1"

[dev-dependencies]
tokio-test = "0.4"
```

### 📅 实施阶段

#### Phase 1: 系统托盘与快捷键 (3-4 小时)
- [ ] 创建 Dioxus 项目框架
- [ ] 实现系统托盘（最小化到托盘）
- [ ] 注册全局快捷键（例如 `Ctrl+Shift+Space` 激活）
- [ ] 浮动输入窗口的基础 UI
- [ ] 窗口管理与置顶功能

#### Phase 2: AI 客户端集成 (4-5 小时)
- [ ] 设计 AI 客户端接口（支持 OpenAI/Anthropic API）
- [ ] 实现 API 调用与流式响应
- [ ] 错误处理与重试机制
- [ ] API Key 管理与配置持久化
- [ ] 响应结果显示（文本格式化）

#### Phase 3: 预配置工具集 (5-6 小时)
- [ ] 实现工具选择器（/summarize, /translate, /explain 等）
- [ ] 开发核心工具：
  - [ ] 文本总结工具
  - [ ] 翻译工具（多语言支持）
  - [ ] 代码解释器
  - [ ] 代码生成器
  - [ ] 重构建议器
- [ ] 上下文获取（从剪贴板、文件选择器）
- [ ] 结果输出（复制到剪贴板、通知）

#### Phase 4: 配置与自定义 (3-4 小时)
- [ ] 用户设置界面
- [ ] 快捷键自定义
- [ ] 工具自定义（添加自定义提示词）
- [ ] AI 模型选择与参数调整
- [ ] 主题系统（暗黑/亮色）

#### Phase 5: 高级功能 (4-5 小时)
- [ ] 对话历史记录
- [ ] 批量处理（选择多个文本）
- [ ] 导出结果（文件、Markdown）
- [ ] 插件系统（扩展工具）
- [ ] 多语言 UI 支持

#### Phase 6: 体验优化 (3-4 小时)
- [ ] 动画效果（窗口切换、加载动画）
- [ ] 音效与触觉反馈
- [ ] 性能优化（响应速度）
- [ ] 快捷操作（Tab 补全、方向键导航）
- [ ] 帮助与文档

#### Phase 7: 测试与打包 (2-3 小时)
- [ ] 多平台测试（Windows/macOS/Linux）
- [ ] 系统托盘图标与通知测试
- [ ] 打包配置（安装程序、自启动）
- [ ] 代码签名（可选）
- [ ] 性能基准测试

### 🎨 UI 设计

#### 浮动输入窗口设计
```rust
// 核心 UI 原则：
// 1. 极简 - 只需一个输入框 + 工具选择
// 2. 快速 - 按下快捷键立即响应（<100ms）
// 3. 非侵入 - 可在任何应用上覆盖显示
// 4. 上下文感知 - 自动从剪贴板或前台应用获取内容

pub struct FloatingInput {
    pub is_visible: bool,
    pub input_text: String,
    pub selected_tool: String,
    pub context: Option<String>,  // 从剪贴板或选择的内容
    pub is_loading: bool,
    pub result: Option<String>,
}

// 工具选择器
pub struct ToolSelector {
    pub tools: Vec<Tool>,  // /summarize, /translate, /explain, /code, /refactor
    pub search_query: String,
}

// 结果查看器
pub struct ResultViewer {
    pub content: String,
    pub can_copy: bool,
    pub can_retry: bool,
}
```

#### 主题色板（简洁现代）
```rust
pub const DARK_THEME: Theme = Theme {
    background: "rgba(15, 23, 42, 0.95)",  // 半透明背景
    surface: "rgba(30, 41, 59, 0.98)",
    primary: "#3b82f6",
    secondary: "#8b5cf6",
    success: "#10b981",
    warning: "#f59e0b",
    error: "#ef4444",
    text: "#f8fafc",
    text_secondary: "#94a3b8",
    border: "rgba(148, 163, 184, 0.2)",
};
```

#### 核心交互流程
1. **激活** (`Ctrl+Shift+Space`) → 浮动窗口出现
2. **选择工具** (`/summarize`, `/translate`, 或输入自定义)
3. **输入内容** (或使用 `Ctrl+V` 从剪贴板粘贴)
4. **执行** (`Enter`) → 发送到 AI API
5. **显示结果** → 复制到剪贴板或通知

### 🔧 核心算法

#### 全局快捷键处理
```rust
// 注册系统级快捷键，支持热重载
fn register_global_shortcuts() -> Result<(), Box<dyn Error>> {
    let mut shortcuts = HotkeyManager::new();

    // 主要激活快捷键
    shortcuts.register("control+shift+space", activate_floating_input)?;

    // 工具快捷键（可选）
    shortcuts.register("control+shift+s", || quick_summarize())?;
    shortcuts.register("control+shift+t", || quick_translate())?;
    shortcuts.register("control+shift+e", || quick_explain())?;

    Ok(())
}
```

#### AI 客户端流式响应
```rust
async fn stream_ai_response(
    client: &OpenAIClient,
    prompt: String,
    tool: &Tool,
) -> Result<String, Box<dyn Error>> {
    // 1. 构建完整提示词（工具 + 用户输入）
    let system_prompt = tool.get_system_prompt();
    let full_prompt = format!("{system_prompt}\n\nUser: {prompt}");

    // 2. 发送流式请求
    let mut stream = client.chat_completion_stream(full_prompt).await?;

    // 3. 实时更新 UI（逐字显示）
    let mut result = String::new();
    while let Some(chunk) = stream.next().await {
        let delta = chunk?.choices[0].delta.content.unwrap_or_default();
        result.push_str(&delta);
        update_ui(&result);  // 实时渲染
    }

    Ok(result)
}
```

#### 上下文自动获取
```rust
// 智能获取上下文（优先级：剪贴板 > 选中文本 > 前台窗口内容）
async fn get_context() -> Option<String> {
    // 1. 检查剪贴板
    if let Ok(text) = get_clipboard() {
        if !text.is_empty() {
            return Some(text);
        }
    }

    // 2. 获取前台窗口选中文本（需要平台特定 API）
    // 3. 获取当前项目文件内容（如果支持）

    None
}
```

### 🚀 性能优化

1. **窗口预热**: 启动时预创建窗口实例，减少激活延迟
2. **API 连接池**: 复用 HTTP 连接，减少 AI API 调用延迟
3. **流式渲染**: 逐字显示 AI 响应，提升感知性能
4. **延迟加载**: 工具配置按需加载（启动时不加载所有工具）
5. **内存缓存**: 缓存常用工具提示词，减少重复构建
6. **本地 LLM 支持**: 可选集成 Ollama，提供离线 AI 能力

### 📦 打包配置

支持多平台打包：
- **Windows**: `.exe` installer + 自启动注册
- **macOS**: `.app` bundle + 辅助功能权限请求
- **Linux**: `.AppImage` 或 `.deb` + 系统托盘支持

### 🧪 测试策略

- **单元测试**: AI 客户端、工具提示词生成
- **集成测试**: 快捷键注册、系统托盘交互
- **UI 测试**: 窗口显示/隐藏、响应速度
- **可用性测试**: 真实用户场景（开发者日常工作流）

### 📚 学习资源

- [Dioxus 官方文档](https://dioxuslabs.com/learn/0.7/)
- [Rust Global Shortcuts](https://docs.rs/winit/*/winit/platform/global_shortcut/)
- [System Tray 指南](https://github.com/tauri-apps/tauri/tree/dev/packages/tauri)
- [OpenAI API 文档](https://platform.openai.com/docs)

### 🎯 里程碑

- **MVP**: 3 天内完成 Phase 1-3（系统托盘 + AI 客户端 + 核心工具）
- **Beta**: 1 周内完成 Phase 4-5（配置 + 高级功能）
- **Release**: 2 周内完成全部功能并打包

### 🔥 核心优势

1. **零学习成本**: 类似 Alfred/PowerToys Run 的交互模式
2. **工作流整合**: 无需切换窗口，直接在当前应用调用 AI
3. **高度可定制**: 工具提示词、快捷键、主题全部可自定义
4. **跨平台**: Windows/macOS/Linux 一套代码
5. **AI 无关**: 支持 OpenAI、Anthropic、本地模型

### 📝 每日开发日志

在 `.claude/.cache/drift-progress.md` 中记录每日进展：
- 完成的功能
- 遇到的问题（系统托盘、快捷键、窗口管理）
- 学习笔记（AI API、平台特定 API）
- 下一步计划

---

**开始日期**: 2025-12-12
**预计完成**: 2025-12-26
**当前状态**: 🟢 已重新设计为 AI 工具包，项目名称为 Veld
