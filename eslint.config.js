import js from "@eslint/js";
import tseslint from "typescript-eslint";
import reactHooks from "eslint-plugin-react-hooks";
import globals from "globals";

export default tseslint.config(
  { ignores: ["dist/**", "coverage/**", "src-tauri/**", "**/node_modules/**", "src-web/lib/jsonvars/*.grammar"] },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    files: ["src-web/**/*.{ts,tsx}"],
    plugins: { "react-hooks": reactHooks },
    languageOptions: { globals: { ...globals.browser } },
    rules: {
      // Only the classic hooks rules. The plugin's React Compiler rules
      // (refs, purity, set-state-in-effect) flag patterns this codebase uses
      // on purpose, such as keeping the latest callback in a ref.
      "react-hooks/rules-of-hooks": "error",
      "react-hooks/exhaustive-deps": "warn",
      "@typescript-eslint/no-unused-vars": ["warn", { argsIgnorePattern: "^_", varsIgnorePattern: "^_" }],
    },
  },
);
