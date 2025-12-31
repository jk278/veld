# Dioxus 热重载完整指南

## 概述

Dioxus 0.7 提供三种热重载方式：

| 类型 | 触发方式 | 稳定性 | 适用场景 |
|------|---------|--------|---------|
| **RSX Hot-Reload** | 保存文件 | ✅ 稳定 | UI 结构、简单样式 |
| **Rust Hot-Patching** | `--hotpatch` | ⚠️ 实验性 | 函数体逻辑（受限） |
| **Asset Hot-Reload** | 保存文件 | ✅ 稳定 | CSS、图片等资源 |

---

## 一、Hot-Patch 已知问题

### 1.1 崩溃模式

| 错误代码 | 含义 | 触发条件 |
|---------|------|---------|
| `0xc00000fd` | 堆栈溢出 | hot-patch 解析器递归深度超限 |
| `0xc0000005` | 访问冲突 | hot-patch 内存指针损坏 |

### 1.2 触发条件

**复杂 Tailwind 变体语法**：

```rust
// ❌ 高风险：嵌套变体 + 长组合
"hover:bg-gray-200 dark:hover:bg-gray-700 border border-transparent hover:border-border"

// ❌ 高风险：多重状态组合
"text-text-secondary hover:text-text-primary hover:bg-gray-200 dark:hover:bg-gray-700"
```

**关键发现**：
- 即使遵循官方规范（移出 rsx!），仍可能崩溃
- 正常 `cargo run` 完全稳定 → **代码无问题，hot-patch 有 bug**
- 这是 Dioxus 0.7 hot-patch 的内在限制

### 1.3 相关 GitHub Issues

- [#2994: Nested RSX does not hot reload](https://github.com/DioxusLabs/dioxus/issues/2994)
- [#3459: Component hot reloading bug](https://github.com/DioxusLabs/dioxus/issues/3459)
- [#3013: Tailwind styles not working](https://github.com/DioxusLabs/dioxus/issues/3013)
- [#2805: Tailwindcss with desktop fails](https://github.com/DioxusLabs/dioxus/issues/2805)

---

## 二、RSX Hot-Reload 支持范围

### 2.1 ✅ 支持

| 类型 | 示例 |
|------|------|
| 元素结构 | 添加/删除/修改元素 |
| 字符串属性 | `class="text-red"` |
| Rust 字面量 | `width: 100`, `enabled: true` |
| 简单表达式 | 变量引用 `{name}` |

### 2.2 ❌ 不支持

| 类型 | 示例 | 替代方案 |
|------|------|---------|
| 复杂表达式 | `format!("{}", x)` | rsx! 外预先计算 |
| 函数调用 | `calc(x * 2)` | 预先计算结果 |
| 新变量/表达式 | 首次引入的变量 | 重启应用 |
| 组件签名变更 | 添加/删除 props | 重启应用 |
| RSX 外逻辑 | 函数体、hooks | 使用 hot-patch 或重启 |

---

## 三、核心原则

> **"复杂表达式移出 `rsx!`，在 Rust 中预先计算"**

这是避免热重载崩溃的**必要但不充分**条件。

### 3.1 三层防御策略

```
┌────────────────────────────────────┐
│   稳定性层级                         │
├────────────────────────────────────┤
│  Level 1: 预先计算 (必要)            │
│  ├─ format! 移出 rsx!               │
│  └─ 条件选择静态字符串                │
│                                    │
│  Level 2: 简化语法 (推荐)            │
│  ├─ 避免复杂 Tailwind 变体           │
│  └─ 使用 CSS 变量代替内联类           │
│                                    │
│  Level 3: 禁用 hot-patch (最稳定)    │
│  └─ 使用 cargo run                  │
└────────────────────────────────────┘
```

---

## 四、解决方案

### 4.1 方案 1: 预先计算（必要）

```rust
// ❌ 错误：format! 在 rsx! 内
rsx! {
  div { class: format!("text-{}", color) }
}

// ✅ 正确：在 rsx! 外计算
let class = format!("text-{}", color);
rsx! {
  div { class: "{class}" }
}
```

### 4.2 方案 2: 条件字符串

```rust
// ✅ 使用 if/else 选择静态字符串
let class = if is_active {
  "bg-blue-500 text-white"
} else {
  "bg-gray-200 text-gray-800"
};
rsx! {
  div { class: "{class}" }
}
```

### 4.3 方案 3: CSS 变量代替复杂类

将复杂 Tailwind 样式移到 CSS 层，使用 `@apply` 或自定义类名，在 Rust 代码中只引用简短的类名。

### 4.4 方案 4: 完全禁用 hot-patch（推荐）

```bash
# 开发时使用标准启动
cargo run

# 发布构建
cargo build --release
```

**优点**：
- ✅ 100% 稳定
- ✅ 避免所有 hot-patch bug
- ✅ 适合生产环境

---

## 五、实践示例

### 5.1 Button 组件（标准模式）

```rust
#[component]
fn Button(
  #[props(default)] variant: ButtonVariant,
  #[props(default)] disabled: bool,
  children: Element,
) -> Element {
  // NOTE: 计算移出 rsx! 避免 Dioxus 热重载 bug
  let base_class = "px-4 py-2 rounded font-medium transition-colors";
  let variant_class = match variant {
    ButtonVariant::Primary => "bg-blue-600 text-white hover:bg-blue-700",
    ButtonVariant::Secondary => "bg-gray-200 text-gray-900 hover:bg-gray-300",
    ButtonVariant::Danger => "bg-red-600 text-white hover:bg-red-700",
  };
  let disabled_class = if disabled { "opacity-50 cursor-not-allowed" } else { "" };
  let full_class = format!("{} {} {}", base_class, variant_class, disabled_class);

  rsx! {
    button {
      class: "{full_class}",
      disabled: "{disabled}",
      {children}
    }
  }
}
```

### 5.2 导航组件

**高风险代码**（来自 `src/components/title_bar.rs`）：

```rust
// ❌ 可能触发 hot-patch 崩溃
let nav_class = if is_active {
  "text-primary bg-bg-surface border border-border/50 hover:bg-bg-tertiary/80 hover:border-border"
} else {
  "text-text-secondary hover:text-text-primary hover:bg-gray-200 dark:hover:bg-gray-700"
};
```

**优化方向**：将复杂 Tailwind 变体移到 CSS 层，Rust 代码只引用简短类名。

---

## 六、调试清单

遇到热重载问题时，按顺序检查：

- [ ] 正常 `cargo run` 能否启动？
  - ✅ 能启动 → hot-patch bug，使用方案 4
  - ❌ 崩溃 → 代码逻辑问题

- [ ] 检查 class 字符串
  - 是否包含复杂 Tailwind 变体（`dark:hover:`）
  - 是否包含长组合（>5 个类）

- [ ] 检查 rsx! 内是否有复杂表达式
  - `format!` 调用
  - 函数调用
  - 复杂计算

- [ ] 检查主题系统
  - `use_memo` 中是否有副作用（如 `document::eval`）
  - 是否存在循环依赖

---

## 七、参考资料

### 官方文档
- [Dioxus Hot-Reload 官方文档](https://dioxuslabs.com/learn/0.7/essentials/ui/hotreload/)

### 相关 Issues
- [#3087: 字符串内容不触发重载](https://github.com/DioxusLabs/dioxus/issues/3087)
- [#3459: 组件热重载 bug](https://github.com/DioxusLabs/dioxus/issues/3459)
- [#2994: Nested RSX hot reload](https://github.com/DioxusLabs/dioxus/issues/2994)

### 项目相关
- `src/components/title_bar.rs` - Hot-patch 崩溃案例
- `src/theme.rs` - 主题系统实现（正确使用 use_memo/use_effect）
