/** @type {import('tailwindcss').Config} */
export default {
  // NOTE: Tailwind CSS v4 扫描器对 Rust `class: "..."` 语法支持有限
  // - 能识别大部分静态类名，但可能遗漏动态构造的类
  // - 当前 CSS: 64KB (256 个类选择器)，包含部分未使用的工具类
  // - 验证: 确认生成了代码中使用的类（如 .max-w-4xl, .bg-bg-surface, .flex）
  //
  // 优化方向:
  // 1. 使用 Tailwind v4 的 @source 指令精确控制扫描范围
  // 2. 考虑将复杂样式移到 input.css 的 @layer components 中
  // 3. 监控 Tailwind Labs 对 Rust 语法的官方支持改进
  //
  // 参考: https://tailwindcss.com/docs/detecting-classes-in-source-files
  content: [
    "./index.html",
    "./src/**/*.{rs,html,css}",
  ],
}
