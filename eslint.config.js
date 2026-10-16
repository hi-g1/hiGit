import { defineConfig } from "eslint/config";
import js from "@eslint/js";
import tseslint from "typescript-eslint";
import reactHooks from "eslint-plugin-react-hooks";
import globals from "globals";

export default defineConfig(
  { ignores: ["dist/", "src-tauri/", "src/lib/bindings.ts"] },
  js.configs.recommended,
  tseslint.configs.recommended,
  reactHooks.configs.flat.recommended,
  {
    files: ["src/**/*.{ts,tsx}"],
    languageOptions: { globals: globals.browser },
    rules: {
      // 저장소에서 온 글자는 React 글자로만 그린다(D-037). HTML로 해석되는 길과 eval을 막는다.
      "no-eval": "error",
      "no-implied-eval": "error",
      "no-new-func": "error",
      "no-script-url": "error",
      "no-restricted-syntax": [
        "error",
        ...[
          "JSXAttribute[name.name='dangerouslySetInnerHTML']",
          "MemberExpression[property.name=/^(innerHTML|outerHTML)$/]",
          "CallExpression[callee.property.name='insertAdjacentHTML']",
          "CallExpression[callee.object.name='document'][callee.property.name=/^write(ln)?$/]",
        ].map((selector) => ({
          selector,
          message: "저장소에서 온 글자는 React로 그대로 그립니다(D-037).",
        })),
        {
          selector: "CallExpression[callee.property.name=/^toLocale(Date|Time)?String$/]",
          message: "날짜·시각 글자는 src/lib/time.ts로 만듭니다(D-150).",
        },
      ],
      // 날짜·시각 형식은 src/lib/time.ts, OS 언어는 src/lib/locale.ts에서만(D-149, D-150).
      "no-restricted-properties": [
        "error",
        { object: "Intl", property: "DateTimeFormat", message: "날짜·시각 글자는 src/lib/time.ts로 만듭니다(D-150)." },
        { object: "Intl", property: "RelativeTimeFormat", message: "날짜·시각 글자는 src/lib/time.ts로 만듭니다(D-150)." },
        { object: "navigator", property: "language", message: "OS 언어는 src/lib/locale.ts에서만 읽습니다(D-149)." },
        { object: "navigator", property: "languages", message: "OS 언어는 src/lib/locale.ts에서만 읽습니다(D-149)." },
        { object: "window", property: "localStorage", message: "웹뷰 저장소는 쓰지 않습니다. 남길 것은 core에 둡니다(D-211)." },
        { object: "window", property: "sessionStorage", message: "웹뷰 저장소는 쓰지 않습니다. 남길 것은 core에 둡니다(D-211)." },
      ],
      "no-restricted-globals": [
        "error",
        { name: "localStorage", message: "웹뷰 저장소는 쓰지 않습니다. 남길 것은 core에 둡니다(D-211)." },
        { name: "sessionStorage", message: "웹뷰 저장소는 쓰지 않습니다. 남길 것은 core에 둡니다(D-211)." },
      ],
      // 화면은 백엔드를 src/lib/를 통해서만 부른다(D-010). invoke를 직접 쓰지 않는다.
      "no-restricted-imports": [
        "error",
        {
          paths: [
            {
              name: "@tauri-apps/api/core",
              message: "백엔드 호출은 src/lib/api.ts를 통해서만 합니다.",
            },
            {
              name: "@tauri-apps/api/event",
              message: "백엔드 이벤트는 src/lib/events.ts를 통해서만 구독합니다.",
            },
          ],
        },
      ],
    },
  },
  {
    files: ["src/lib/**/*.{ts,tsx}"],
    rules: { "no-restricted-imports": "off" },
  },
  {
    files: ["src/lib/time.ts", "src/lib/locale.ts", "src/lib/legacyStorage.ts", "src/**/*.test.{ts,tsx}"],
    rules: { "no-restricted-properties": "off", "no-restricted-globals": "off" },
  },
  {
    files: ["*.config.{js,ts}", "scripts/**/*.mjs"],
    languageOptions: { globals: globals.node },
  },
);
