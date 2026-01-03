# Veld 发布步骤清单

> 从开发到公开发布的完整流程

## 前置准备

### 代码状态检查

- [ ] 移除调试日志（`info!`, `debug!` 等）
- [ ] 更新版本号（`Cargo.toml` → `version = "0.0.1"`）
- [ ] 确认 README.md 信息完整
- [ ] 测试核心功能无崩溃

### 版本号规范

```
0.0.1 - 首个公开版本，功能不完整
0.1.0 - Alpha，核心功能可用
0.2.0 - Beta，功能完整，可能有小 bug
1.0.0 - 稳定版，生产可用
```

---

## 第一步：本地测试构建

### Windows 打包测试

```bash
# 1. 清理旧构建
cargo clean

# 2. 编译 Tailwind CSS
pnpm tailwind:build

# 3. 构建发布版本
cargo build --release

# 4. 测试运行
./target/release/veld.exe

# 5. 手动打包
mkdir release
copy target\release\veld.exe release\
copy README.md release\
copy RELEASE_NOTES.md release\
powershell Compress-Archive -Path release\* -DestinationPath veld-windows-x64.zip
```

### 验证清单

- [ ] 程序能正常启动
- [ ] 系统托盘图标显示
- [ ] 全局快捷键 `Ctrl+Shift+Space` 响应
- [ ] 配置文件正常读写
- [ ] AI 对话功能正常

---

## 第二步：创建 GitHub Release

### 方式 A：使用 GitHub Actions（推荐）

**已配置文件**：`.github/workflows/release.yml`

```bash
# 1. 提交所有更改
git add .
git commit -m "chore: prepare for v0.0.1 release"

# 2. 创建版本标签
git tag v0.0.1

# 3. 推送代码和标签
git push origin main
git push origin v0.0.1

# 4. 等待 GitHub Actions 完成构建（约 5-10 分钟）

# 5. 在 GitHub Releases 编辑 Draft Release
#    - 上传截图
#    - 补充发布说明
#    - 点击 "Publish" 发布
```

### 方式 B：手动打包上传

```bash
# 1. 本地打包（见第一步）

# 2. 访问 GitHub Releases 页面
# https://github.com/yourusername/veld/releases/new

# 3. 填写信息
# - Tag: v0.0.1
# - Title: Veld v0.0.1
# - Description: 复制 RELEASE_NOTES.md 内容
# - Attachments: 上传 veld-windows-x64.zip

# 4. 点击 "Publish release"
```

---

## 第三步：社交媒体推广

### 推广渠道

#### 1. GitHub 生态

**GitHub Release**
- 完整的 Release Notes
- 截图（2-3 张）
- 安装说明

**GitHub Discussion**
- 标题：🌾 Veld v0.0.1 - First public release
- 内容：简短介绍 + 链接到 Release
- 标签：`announcements`

#### 2. Reddit

**r/rust**
- 标题：`[Release] Veld v0.0.1 - System tray AI assistant built with Rust + Dioxus`
- 内容：技术栈 + 功能特性 + GitHub 链接
- 时间：北京时间上午 9-11 点（美国活跃时间）

**r/ArtificialIntelligence**
- 标题：`I built an AI assistant that lives in your system tray`
- 内容：侧重用户体验 + 使用场景

#### 3. Hacker News

**Show HN**
- 标题：`Show HN: Veld - AI assistant in system tray (Rust + Dioxus)`
- 内容：
  - 一句话介绍
  - 为什么做这个项目
  - 技术亮点
  - GitHub 链接

**发布时间**：美国东部时间上午 9-10 点（北京时间晚上 10-11 点）

#### 4. Twitter/X

```
🌾 I built Veld - a lightweight AI assistant in your system tray

Press Ctrl+Shift+Space → Access AI from anywhere

✅ Rust + Dioxus 0.7
✅ 5+ AI providers (OpenAI, Claude, DeepSeek...)
✅ Streaming responses, MCP integration

v0.0.1 is out! 👇
[GitHub Link]

#rustlang #buildinpublic #AI
```

### 推广时间表

```
Day 0: GitHub Release 发布
Day 1: Reddit r/rust + GitHub Discussion
Day 2: Hacker News Show HN
Day 3: Reddit r/ArtificialIntelligence
Day 4: Twitter/X
```

---

## 第四步：收集反馈

### 反馈渠道

- **GitHub Issues**: Bug 报告
- **GitHub Discussions**: 功能建议
- **社交媒体评论**: 用户体验反馈

### 关注指标

- GitHub Stars 增长
- 下载量（GitHub Release 统计）
- Issue/讨论数量
- 社交媒体互动

### 回复模板

**感谢反馈**
```
Thanks for trying Veld! This is exactly the kind of feedback I'm looking for.

I've added this to the roadmap: [link]
Would you like to share your use case? It would help me prioritize.
```

**Bug 确认**
```
I can reproduce this issue. It's now tracked here: [issue-link]

Workaround: [temporary solution if available]
Fix planned for: [version]
```

---

## 第五步：迭代计划

### v0.0.2 规划

根据 v0.0.1 反馈确定：
- 最需要的功能
- 最严重的 bug
- 最多人请求的 AI 提供商

### 发布节奏建议

```
v0.0.1 → v0.0.2: 1-2 周（修复关键 bug）
v0.0.2 → v0.0.3: 2-3 周（小功能迭代）
v0.0.x → v0.1.0: 1-2 月（功能里程碑）
```

---

## 附录：常用命令

### Git 操作

```bash
# 查看当前标签
git tag

# 删除错误标签（本地）
git tag -d v0.0.1

# 删除错误标签（远程）
git push origin :refs/tags/v0.0.1

# 查看提交历史
git log --oneline -10
```

### Cargo 操作

```bash
# 检查版本号
cargo pkgid

# 更新依赖
cargo update

# 检查编译
cargo check

# 运行测试
cargo test
```

---

## 注意事项

### Build in Public 心态

- **透明度**: 分享开发过程，包括失败
- **真实性**: 诚实地标注当前限制
- **开放性**: 欢迎早期用户参与决策

### 避免过度承诺

- README 中标注 "Alpha" / "Beta" 状态
- Release Notes 列出已知限制
- Roadmap 明确时间表

### 安全检查

- [ ] 移除硬编码的 API Key
- [ ] 检查敏感信息（密码、token）
- [ ] 确认日志不泄露用户数据

---

**发布只是开始，真正的价值在于持续的迭代和社区互动。**
