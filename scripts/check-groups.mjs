// 스테이지된 파일 목록으로 pre-commit이 돌릴 검사 묶음을 고른다(D-022).

/** 프론트 검사(lint, typecheck, vitest)가 필요한 파일 */
export function isFrontend(path) {
  return (
    path.startsWith("src/") ||
    path.startsWith("scripts/") ||
    path === "docs/spec.md" || // 용어표 일치 테스트
    path === "package.json" ||
    path === "pnpm-lock.yaml" ||
    path === "tsconfig.json" ||
    path === "index.html" ||
    path === ".stylelintrc.json" ||
    /^[^/]+\.config\.(js|ts|mjs)$/.test(path)
  );
}

/** Rust 검사(fmt, clippy)가 필요한 파일 */
export function isRust(path) {
  return path.startsWith("src-tauri/") || path === "rust-toolchain.toml";
}

/** @returns {{ frontend: boolean, rust: boolean }} */
export function selectGroups(paths) {
  return {
    frontend: paths.some(isFrontend),
    rust: paths.some(isRust),
  };
}
