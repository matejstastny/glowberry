import js from "@eslint/js";
import ts from "typescript-eslint";
import globals from "globals";

export default ts.config(
    {
        ignores: ["dist/**", "node_modules/**", "src-tauri/**"],
    },
    js.configs.recommended,
    ...ts.configs.recommended,
    {
        languageOptions: {
            globals: {
                ...globals.browser,
            },
        },
    },
    {
        rules: {
            "@typescript-eslint/no-unused-vars": ["warn", { argsIgnorePattern: "^_" }],
            "@typescript-eslint/no-explicit-any": "warn",
        },
    },
);
