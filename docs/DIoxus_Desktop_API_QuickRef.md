# Dioxus 0.7 Desktop API Quick Reference

## 📚 概述

Dioxus Desktop 基于 Wry/TAO，提供了丰富的原生桌面集成 API。避免重复造轮子，先查此文档！

---

## ⌨️ 全局快捷键

### `use_global_shortcut`

注册系统级全局快捷键（即使应用未获得焦点也能触发）。

```rust
use dioxus_desktop::use_global_shortcut;

let _handle = use_global_shortcut(
    "Ctrl+Shift+Space",  // 快捷键字符串
    move |state| {       // 回调函数
        if state == dioxus_desktop::HotKeyState::Pressed {
            // 快捷键被按下时的逻辑
            show_window.set(true);
        }
    },
);
```

**特点**：
- ✅ 在任何应用上都能触发
- ✅ 自动处理事件循环集成
- ✅ 返回 `ShortcutHandle` 用于取消注册
- ✅ 支持多种格式：`"Ctrl+Shift+Space"`, `"Alt+F4"`, `"Cmd+Q"` (macOS)

---

## 🔌 事件循环集成

### `use_wry_event_handler`

直接访问 Wry 事件循环，处理底层系统事件。

```rust
use dioxus_desktop::use_wry_event_handler;
use winit::event::{Event, WindowEvent};

use_wry_event_handler(move |event: &Event<_>, _target| {
    match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            window_id,
            ..
        } => {
            // 处理窗口关闭事件
            println!("Window {:?} close requested", window_id);
        }
        _ => {}
    }
});
```

**常用事件类型**：
- `WindowEvent::CloseRequested` - 窗口关闭请求
- `WindowEvent::Focused(bool)` - 窗口获得/失去焦点
- `WindowEvent::Resized(PhysicalSize)` - 窗口大小改变
- `WindowEvent::Moved(PhysicalPosition)` - 窗口移动
- `WindowEvent::KeyboardInput` - 键盘输入

---

## 📋 系统托盘事件

### `use_tray_icon_event_handler`

监听托盘图标点击事件。

```rust
use dioxus_desktop::use_tray_icon_event_handler;
use tray_icon::TrayIconEvent;

use_tray_icon_event_handler(move |event: TrayIconEvent| {
    match event {
        TrayIconEvent::Click { .. } => {
            // 托盘图标被点击
            show_app_window();
        }
        _ => {}
    }
});
```

### `use_tray_menu_event_handler`

监听托盘菜单事件。

```rust
use dioxus_desktop::use_tray_menu_event_handler;
use muda::MenuEvent;

use_tray_menu_event_handler(move |event: MenuEvent| {
    match event.id.as_str() {
        "show" => show_window(),
        "quit" => quit_app(),
        _ => {}
    }
});
```

---

## 🧭 菜单栏事件

### `use_muda_event_handler`

监听原生菜单栏事件。

```rust
use dioxus_desktop::use_muda_event_handler;
use muda::MenuEvent;

use_muda_event_handler(move |event: MenuEvent| {
    match event.id.as_str() {
        "file_new" => create_new_file(),
        "edit_paste" => paste_clipboard(),
        _ => {}
    }
});
```

---

## 🪟 窗口管理

### 通过 DesktopContext 控制窗口

```rust
use dioxus_desktop::DesktopContext;

fn App() -> Element {
    let desktop_ctx = use_context::<DesktopContext>();

    rsx! {
        button {
            onclick: move |_| {
                // 隐藏窗口
                desktop_ctx.window().set_visible(false);
            },
            "Hide Window"
        }
        button {
            onclick: move |_| {
                // 显示窗口
                desktop_ctx.window().set_visible(true);
            },
            "Show Window"
        }
        button {
            onclick: move |_| {
                // 退出应用
                desktop_ctx.quit();
            },
            "Quit"
        }
    }
}
```

**常用方法**：
- `window().set_visible(bool)` - 显示/隐藏窗口
- `window().set_focus()` - 获得窗口焦点
- `window().set_position(position)` - 移动窗口
- `window().set_size(size)` - 调整窗口大小
- `quit()` - 退出应用

---

## 📊 窗口信息

### `use_wry_window`

获取窗口句柄和控制接口。

```rust
use dioxus_desktop::use_wry_window;

fn App() -> Element {
    let window = use_wry_window();

    rsx! {
        div {
            // 窗口相关操作
            onclick: move |_| {
                window.set_focus().unwrap();
            }
        }
    }
}
```

---

## 📦 导入指南

**核心导入**：
```rust
use dioxus_desktop::{
    use_global_shortcut,  // 全局快捷键
    use_wry_event_handler, // 事件循环
    use_tray_icon_event_handler,    // 托盘图标事件
    use_tray_menu_event_handler,    // 托盘菜单事件
    use_muda_event_handler,         // 菜单栏事件
    use_wry_window,      // 窗口句柄
    DesktopContext,      // 桌面上下文
    HotKeyState,         // 快捷键状态
};
```

---

## ⚠️ 重要提醒

1. **优先使用内置 API**：dioxus-desktop 已经封装了大部分原生功能，无需手动引入 `global-hotkey`、`winit` 等依赖
2. **事件循环自动管理**：所有 Hook 都会自动在正确的时机注册/注销
3. **返回值的处理**：返回的 `Handle` 类型会自动管理生命周期，无需手动保存
4. **线程安全**：所有 API 都可以在组件的任意位置使用，无需考虑线程问题

---

## 📚 相关资源

- [Dioxus Desktop 完整文档](https://docs.rs/dioxus-desktop/0.7.2/dioxus_desktop)
- [Wry API 文档](https://docs.rs/wry/0.30.12/wry)
- [TAO 窗口管理](https://docs.rs/tao/0.34/tao)

---

## 🚀 示例项目

查看完整示例：
- [系统托盘应用](https://github.com/DioxusLabs/dioxus/tree/master/examples/desktop_tray)
- [全局快捷键](https://github.com/DioxusLabs/dioxus/tree/master/examples/desktop_shortcuts)
- [菜单栏](https://github.com/DioxusLabs/dioxus/tree/master/examples/desktop_menu)

