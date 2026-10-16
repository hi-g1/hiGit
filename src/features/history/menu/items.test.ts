import { QueryClient } from "@tanstack/react-query";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("../../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../../lib/api")>()),
  api: { executeAction: vi.fn() },
}));

import { api } from "../../../lib/api";
import actionMeta from "../../../lib/action-meta.json";
import { ko } from "../../../messages/ko";
import type { StationInfo } from "./info";
import { STATION_ITEMS, type MenuCtx, type MenuEnv, type StationMenuItem } from "./items";

const oid = "a".repeat(40);
const info = (over: Partial<StationInfo> = {}): StationInfo => ({
  oid,
  shortOid: "aaaaaaa",
  summary: "버전",
  author: "지원",
  time: 0,
  parents: ["b".repeat(40)],
  isMerge: false,
  isRoot: false,
  isHead: false,
  onHeadLine: true,
  isAncestorOfHead: true,
  branches: [],
  tags: [],
  unpushed: true,
  ...over,
});
const env = (over: Partial<MenuEnv> = {}): MenuEnv => ({
  repo: "/r",
  currentBranch: "main",
  changedFiles: 0,
  stagedFiles: 0,
  operation: "none",
  upstream: null,
  defaultBranch: "main",
  zoom: "all",
  headLine: [],
  ...over,
});
const item = (id: string) => STATION_ITEMS.find((i) => i.id === id)!;
/** "show"·"hide"·흐린 까닭 글자. */
const state = (id: string, i: StationInfo, e: MenuEnv) => {
  const a = item(id).availability(i, e);
  return typeof a === "object" ? a.disabled(ko) : a;
};

afterEach(() => vi.clearAllMocks());

describe("되돌리기·복사·돌아가기 항목", () => {
  it.each([
    // [항목, 역, 화면, 기대]
    ["revert", info(), env(), "show"],
    ["revert", info({ isHead: true }), env(), "show"],
    ["revert", info({ isAncestorOfHead: false }), env(), "hide"],
    ["revert", info(), env({ stagedFiles: 1 }), ko.whyStaged],
    ["revert", info(), env({ operation: "other" }), ko.whyOtherOperation],
    ["revert", info(), env({ currentBranch: null }), ko.whyDetached],
    ["cherry-pick", info(), env(), "hide"],
    ["cherry-pick", info({ isAncestorOfHead: false, onHeadLine: false }), env(), "show"],
    ["cherry-pick", info({ isAncestorOfHead: false, isMerge: true }), env(), ko.whyMergeCopy],
    ["cherry-pick", info({ isAncestorOfHead: false }), env({ stagedFiles: 2 }), ko.whyStaged],
    ["cherry-pick", info({ isAncestorOfHead: false }), env({ operation: "other" }), ko.whyOtherOperation],
    ["reset", info(), env(), "show"],
    ["reset", info({ isHead: true }), env(), "hide"],
    ["reset", info({ isAncestorOfHead: false }), env(), "hide"],
    ["reset", info(), env({ changedFiles: 3 }), "show"],
    ["reset", info(), env({ operation: "other" }), ko.whyOtherOperation],
    ["reset", info(), env({ currentBranch: null }), ko.whyDetached],
    ["new-branch", info(), env(), "show"],
    ["new-branch", info(), env({ changedFiles: 1 }), ko.whyChanged],
    ["new-branch", info({ isHead: true }), env({ changedFiles: 1 }), "show"],
    ["new-branch", info(), env({ operation: "other" }), ko.whyOtherOperation],
    ["tag", info({ isAncestorOfHead: false }), env({ currentBranch: null }), "show"],
    ["files", info({ isAncestorOfHead: false }), env({ operation: "other", currentBranch: null }), "show"],
    ["rewrite-from", info(), env({ headLine: ["h".repeat(40), oid] }), "show"],
    ["rewrite-from", info({ isHead: true }), env({ headLine: [oid] }), "hide"],
    ["rewrite-from", info({ onHeadLine: false }), env({ headLine: ["h".repeat(40), oid] }), "hide"],
    ["rewrite-from", info({ isMerge: true }), env({ headLine: ["h".repeat(40), oid] }), "hide"],
    ["rewrite-from", info(), env({ headLine: [...Array.from({ length: 50 }, (_, i) => `${i}`.padStart(40, "0")), oid] }), "hide"],
    ["rewrite-from", info(), env({ headLine: ["h".repeat(40), oid], zoom: "key" }), ko.whyZoomAll],
    ["rewrite-from", info(), env({ headLine: ["h".repeat(40), oid], changedFiles: 1 }), ko.whyChanged],
    ["rewrite-from", info(), env({ headLine: ["h".repeat(40), oid], operation: "other" }), ko.whyOtherOperation],
    ["tag", info(), env({ operation: "other" }), ko.whyOtherOperation],
    ["amend", info({ isHead: true }), env(), "show"],
    ["amend", info(), env(), "hide"],
    ["amend", info({ isHead: true, unpushed: false }), env({ upstream: "origin/main" }), ko.whyPublished],
    ["amend", info({ isHead: true, unpushed: false }), env(), "show"],
    ["amend", info({ isHead: true }), env({ operation: "other" }), ko.whyOtherOperation],
    ["amend", info({ isHead: true }), env({ currentBranch: null }), ko.whyDetached],
  ] as const)("%s", (id, i, e, expected) => {
    expect(state(id, i, e)).toBe(expected);
  });

  it.each([
    ["revert", { kind: "reverted", shortOid: "c", summary: "x" }, ko.reverted],
    ["cherry-pick", { kind: "picked", shortOid: "c", summary: "x" }, ko.picked],
    ["reset", { kind: "reset", dropped: 3 }, ko.resetDone(3)],
  ] as const)("%s는 그 액션에 해시를 담아 보내고 결과를 알린다", async (id, result, text) => {
    vi.mocked(api.executeAction).mockResolvedValue(result);
    const ctx: MenuCtx = {
      repo: "/r",
      toast: vi.fn(),
    toastError: vi.fn(),
      queryClient: new QueryClient(),
      onDone: vi.fn(),
      ask: vi.fn(),
      openActivity: vi.fn(),
      openRewrite: vi.fn(),
      headLine: () => [],
      m: ko,
    };
    await item(id).run(info(), ctx);
    expect(api.executeAction).toHaveBeenCalledWith("/r", { id, input: { oid } });
    expect(ctx.onDone).toHaveBeenCalledWith(text);
  });
});

/**
 * 되돌릴 수 없는(irreversible) 액션을 부르면서 확인 표시(`confirms`)가 없는 항목 id(D-043).
 * 그런 항목은 `run`이 바로 실행하지 않고 `ctx.ask`로 확인 대화상자를 연다.
 */
function unconfirmed(items: readonly Pick<StationMenuItem, "id" | "actionId" | "confirms">[]): string[] {
  const meta = new Map(actionMeta.map((a) => [a.id, a]));
  return items.filter((i) => i.actionId && meta.get(i.actionId)?.undo === "irreversible" && !i.confirms).map((i) => i.id);
}

describe("항목 표 가드", () => {
  it("액션 항목은 action-meta에 있다", () => {
    const meta = new Map(actionMeta.map((a) => [a.id, a]));
    for (const i of STATION_ITEMS) {
      if (i.actionId) expect(meta.get(i.actionId), i.id).toBeDefined();
    }
  });

  it("되돌릴 수 없는 액션 항목은 확인 대화상자를 거친다", async () => {
    expect(unconfirmed(STATION_ITEMS)).toEqual([]);
    for (const i of STATION_ITEMS.filter((x) => x.confirms)) {
      const ask = vi.fn();
      const executeAction = vi.mocked(api.executeAction);
      executeAction.mockClear();
      await i.run(info(), { ask } as unknown as MenuCtx);
      expect(ask, i.id).toHaveBeenCalledTimes(1);
      expect(executeAction, i.id).not.toHaveBeenCalled();
    }
  });

  it("가드는 확인 표시 없이 넣은 되돌릴 수 없는 액션을 잡는다", () => {
    const irreversible = actionMeta.find((a) => a.undo === "irreversible");
    expect(irreversible, "action-meta에 되돌릴 수 없는 액션이 있다(버리기 등)").toBeDefined();
    expect(unconfirmed([{ id: "x", actionId: irreversible!.id }])).toEqual(["x"]);
    expect(unconfirmed([{ id: "x", actionId: irreversible!.id, confirms: true }])).toEqual([]);
  });
});

describe("대화상자를 여는 항목", () => {
  it.each([
    ["tag", "tag"],
    ["new-branch", "newBranch"],
    ["amend", "amend"],
    ["files", "files"],
  ] as const)("%s는 대화상자를 연다", (id, kind) => {
    const ask = vi.fn();
    const ctx = { ask } as unknown as MenuCtx;
    void item(id).run(info(), ctx);
    expect(ask).toHaveBeenCalledWith({ kind, info: info() });
  });
});

describe("여기부터 기록 정리", () => {
  it("그 역부터 지금 위치까지(새 것부터)로 화면을 연다", () => {
    const openRewrite = vi.fn();
    const line = ["h".repeat(40), "g".repeat(40), oid, "b".repeat(40)];
    const ctx = { openRewrite, headLine: () => line } as unknown as MenuCtx;
    void item("rewrite-from").run(info(), ctx);
    expect(openRewrite).toHaveBeenCalledWith(["h".repeat(40), "g".repeat(40), oid]);
  });
});
