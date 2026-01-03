# Veld v0.0.1 - First Public Release 🌾

> Where ideas roam free

## 🎯 What is Veld?

Veld is a lightweight AI assistant that lives in your system tray. Press `Ctrl+Shift+Space` to instantly access AI help from anywhere on your computer.

## ✨ Current Features

### Core Functionality
- **Global Hotkey**: Press `Ctrl+Shift+Space` to activate
- **Multiple AI Providers**: OpenAI, Anthropic, DeepSeek, GLM (智谱), MiniMax
- **Streaming Responses**: Real-time AI output
- **Session Management**: Chat history with search
- **MCP Integration**: Extendable tool system

### Quick Tools
- `/summarize` - Summarize content
- `/explain` - Explain code or concepts
- `/translate` - Translate to English
- `/refactor` - Refactoring suggestions
- `/doc` - Generate documentation
- `/test` - Generate unit tests

### UI Features
- Dark/Light/System theme
- Syntax highlighting for code
- Responsive design
- Minimize to tray

## 🛠️ Tech Stack

- **Frontend**: Dioxus 0.7 (Rust + Web)
- **AI Client**: rig-core 0.27
- **Styling**: Tailwind CSS
- **Markdown**: pulldown-cmark
- **Code Highlight**: syntect

## 🚧 Known Limitations

- Memory feature not yet implemented
- Only Windows builds available (macOS/Linux coming soon)
- Requires API keys for AI providers

## 🗺️ Roadmap

- [ ] Memory system (RAG-based)
- [ ] Workflow automation
- [ ] Voice input
- [ ] macOS/Linux builds
- [ ] Plugin system

## 💬 Feedback Wanted

This is an **early alpha** release. I'm building in public and would love your feedback:

- What features would you use daily?
- Any bugs or rough edges?
- Which AI providers do you want added?

Join the discussion: [GitHub Issues](https://github.com/yourusername/veld/issues)

## 📦 Installation

Download the latest release from [GitHub Releases](https://github.com/yourusername/veld/releases)

1. Extract `veld.zip`
2. Run `veld.exe`
3. Configure your API key in Settings
4. Press `Ctrl+Shift+Space` to start!

---

**Built with ❤️ using Rust and Dioxus**
