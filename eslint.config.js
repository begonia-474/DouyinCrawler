import js from "@eslint/js";
import tseslint from "typescript-eslint";
import reactHooks from "eslint-plugin-react-hooks";

export default tseslint.config(
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    plugins: {
      "react-hooks": reactHooks,
    },
    rules: {
      // React hooks 规则 — 捕捉 effect 依赖问题
      "react-hooks/rules-of-hooks": "error",
      "react-hooks/exhaustive-deps": "warn",
      // 允许 _ 前缀的未使用变量（解构丢弃、函数参数等）
      "@typescript-eslint/no-unused-vars": ["error", { argsIgnorePattern: "^_", varsIgnorePattern: "^_" }],
    },
  },
  {
    ignores: [
      "dist/**",
      "src-tauri/**",
      "node_modules/**",
      "core/**",
      "backend/**",
      "scripts/**",
      "*.config.*",
    ],
  }
);
