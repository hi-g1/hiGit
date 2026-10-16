import { describe, expect, it } from "vitest";
import type { BranchListView, BranchRow } from "../../lib/api";
import { branchColor } from "./color";
import { canOfferCreate, rightText, search, sections } from "./model";

const row = (name: string, over: Partial<BranchRow> = {}): BranchRow => ({
  name,
  remote: null,
  isHead: false,
  isDefault: false,
  upstream: null,
  ahead: 0,
  behind: 0,
  gone: false,
  lastAuthor: "민지",
  lastAt: 100,
  parkedCount: 0,
  parkBlocked: false,
  ...over,
});

const view: BranchListView = {
  local: [
    row("로그인 기능", { isHead: true }),
    row("main", { isDefault: true, lastAt: 500, behind: 1 }),
    ...["a", "b", "c", "d", "e", "f"].map((n, i) => row(n, { lastAt: 400 - i })),
  ],
  remote: [row("origin/main", { remote: "origin" }), row("origin/원격만", { remote: "origin" })],
  defaultBranch: "main",
  orphanParked: [],
  remotes: ["origin"],
};

describe("브랜치 목록 모델", () => {
  it("고정한 브랜치는 맨 위 묶음, 지금 브랜치와 없는 이름은 빼고 고른 순서대로(D-203)", () => {
    const s = sections(view, ["c", "없음", "로그인 기능", "main"]);
    expect(s.pinned.map((i) => i.name)).toEqual(["c", "main"]);
    expect(s.current?.name).toBe("로그인 기능");
    expect(s.recent.map((i) => i.name)).toEqual(["a", "b", "d", "e", "f"]);
  });

  it("지금, 최근 5개, 나머지(원격에만 있는 것 포함)", () => {
    const s = sections(view);
    expect(s.current?.name).toBe("로그인 기능");
    expect(s.recent.map((i) => i.name)).toEqual(["main", "a", "b", "c", "d"]);
    expect(s.more.map((i) => [i.name, i.remote])).toEqual([
      ["e", null],
      ["f", null],
      ["원격만", "origin"],
    ]);
  });

  it("찾기는 대소문자를 가리지 않고, 같은 이름이 없을 때만 새로 만들기", () => {
    expect(search(view, "MA").map((i) => i.name)).toEqual(["main"]);
    expect(search(view, "원격").map((i) => i.remote)).toEqual(["origin"]);
    expect(canOfferCreate(view, "결제 화면")).toBe(true);
    expect(canOfferCreate(view, "MAIN")).toBe(false);
    expect(canOfferCreate(view, "  ")).toBe(false);
  });

  it("오른쪽 글자 우선순위", () => {
    expect(rightText(row("x", { parkedCount: 2, parkBlocked: true, behind: 3 }))).toEqual({ kind: "parkBlocked", count: 2 });
    expect(rightText(row("x", { parkedCount: 2, behind: 3 }))).toEqual({ kind: "parked", count: 2 });
    expect(rightText(row("x", { behind: 3 }))).toEqual({ kind: "newVersions", count: 3 });
    expect(rightText(row("x"))).toEqual({ kind: "author", author: "민지", at: 100 });
  });

  it("색: 기본 브랜치는 1번, 같은 이름은 늘 같은 색", () => {
    expect(branchColor("main", "main")).toBe("var(--graph-1)");
    expect(branchColor("기능", "main")).toBe(branchColor("기능", "main"));
    expect(branchColor("기능", "main")).toMatch(/var\(--graph-[2-6]\)/);
  });
});
