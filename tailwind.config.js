/** @type {import('tailwindcss').Config} */
export default {
  // NOTE: Tailwind CSS 扫描器无法识别 .rs 文件中的 rsx 语法（class: "..."）
  // 因此无法按需生成 CSS，生成了完整的 ~57KB tailwind.css
  // 这是当前 Dioxus + Tailwind 的已知限制，需等待官方支持
  content: [
    "./index.html",
    "./src/**/*.{rs,html,css}",
  ],
}
