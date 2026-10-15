// @vitest-environment node
import { describe, expect, it } from "vitest";
import { selectGroups } from "./check-groups.mjs";

describe("selectGroups", () => {
  it("프론트만 바뀌면 프론트 검사만", () => {
    expect(selectGroups(["src/App.tsx", "src/styles/tokens.css"])).toEqual({ frontend: true, rust: false });
  });

  it("Rust만 바뀌면 Rust 검사만", () => {
    expect(selectGroups(["src-tauri/crates/git/src/lib.rs"])).toEqual({ frontend: false, rust: true });
  });

  it("둘 다 바뀌면 둘 다", () => {
    expect(selectGroups(["src/lib/bindings.ts", "src-tauri/src/commands.rs"])).toEqual({
      frontend: true,
      rust: true,
    });
  });

  it("문서만 바뀌면 검사하지 않는다", () => {
    expect(selectGroups(["docs/roadmap.md", "CLAUDE.md", "README.md"])).toEqual({
      frontend: false,
      rust: false,
    });
  });

  it("용어표와 설정 파일은 프론트 검사", () => {
    expect(selectGroups(["docs/spec.md"]).frontend).toBe(true);
    expect(selectGroups(["vite.config.ts"]).frontend).toBe(true);
    expect(selectGroups(["rust-toolchain.toml"]).rust).toBe(true);
  });
});
