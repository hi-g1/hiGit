// @vitest-environment node
import { describe, expect, it } from "vitest";
import type { ActionMetaView } from "../../lib/api";
import meta from "../../lib/action-meta.json";
import { ko } from "../../messages/ko";
import { DIMMED, EXCLUDED, OPEN_UI, RUN_DIRECT, buildCommands, searchCommands, type PaletteInput } from "./commands";

const actions = meta as ActionMetaView[];

const input = (over: Partial<PaletteInput> = {}): PaletteInput => ({
  actions,
  locale: "ko",
  m: ko,
  hasRepo: true,
  changedFiles: 2,
  hasRemote: true,
  hasUpstream: true,
  merging: false,
  current: "main",
  branches: [
    { name: "main", remote: null },
    { name: "디자인-수정", remote: null },
    { name: "원격기능", remote: "origin" },
  ],
  recent: [
    { path: "/home/me/blog", name: "blog", lastOpenedAt: 1 , pinned: false, pinnedBranches: []},
    { path: "/repo", name: "repo", lastOpenedAt: 2 , pinned: false, pinnedBranches: []},
  ],
  repoPath: "/repo",
  ...over,
});

describe("명령 팔레트 목록(D-199)", () => {
  it("모든 액션이 분류표 한 곳에만 있다", () => {
    const tables = [RUN_DIRECT, OPEN_UI, DIMMED, EXCLUDED].map((t) => new Set(Object.keys(t)));
    for (const { id } of actions) {
      expect(tables.filter((t) => t.has(id)).length, id).toBe(1);
    }
    // 표에만 있고 없는 액션도 없다.
    const ids = new Set(actions.map((a) => a.id));
    for (const t of tables) for (const id of t) expect(ids.has(id), id).toBe(true);
  });

  it("바로 하기는 입력 없이 core에 보내거나 고정 입력이고, 되돌릴 수 없는 것은 없다", () => {
    for (const id of Object.keys(RUN_DIRECT)) {
      const m = actions.find((a) => a.id === id)!;
      expect(m.undo, id).not.toBe("irreversible");
      expect(["safe", "normal"]).toContain(m.risk);
    }
    for (const m of actions.filter((a) => a.undo === "irreversible")) {
      expect(RUN_DIRECT[m.id], m.id).toBeUndefined();
    }
    // 입력이 필요한 액션은 고정 입력이 있는 바로 하기, 화면 열기, 흐림, 제외 중 하나다.
    const cmds = buildCommands(input());
    for (const c of cmds.filter((c) => c.kind === "run")) {
      if (c.kind !== "run") continue;
      expect(c.input.id).toBe(c.key.slice(4));
    }
  });

  it("상태에 따라 흐리고 까닭을 단다", () => {
    const find = (cmds: ReturnType<typeof buildCommands>, key: string) => cmds.find((c) => c.key === key)!;
    let cmds = buildCommands(input({ changedFiles: 0, hasRemote: false, hasUpstream: false }));
    expect(find(cmds, "run:stash").disabledReason).toBe("고친 파일이 없어요");
    expect(find(cmds, "run:sync").disabledReason).toBe("연결된 원격 저장소가 없어요");
    expect(find(cmds, "run:pull").disabledReason).toBe("원격 저장소와 이어지지 않은 브랜치예요");
    expect(find(cmds, "dim:pr-merge").disabledReason).toBe("검토 요청 화면에서 해요");
    cmds = buildCommands(input({ merging: true }));
    expect(find(cmds, "run:sync").disabledReason).toBe("합치기를 마친 뒤에 할 수 있어요");
    expect(find(cmds, "switch:/디자인-수정").disabledReason).toBe("합치기를 마친 뒤에 할 수 있어요");
  });

  it("브랜치·최근 프로젝트는 지금 것을 빼고, 시작 화면은 짧은 목록", () => {
    const keys = buildCommands(input()).map((c) => c.key);
    expect(keys).toContain("switch:/디자인-수정");
    expect(keys).toContain("switch:origin/원격기능");
    expect(keys).not.toContain("switch:/main");
    expect(keys).toContain("recent:/home/me/blog");
    expect(keys).not.toContain("recent:/repo");
    const start = buildCommands(input({ hasRepo: false, repoPath: null })).map((c) => c.key);
    expect(start).toEqual(["view:openFolder", "view:clone", "view:settings", "view:shortcuts", "recent:/home/me/blog", "recent:/repo"]);
  });

  it("초성·섞인 찾기, 이름에 그대로 있으면 먼저", () => {
    const cmds = buildCommands(input());
    expect(searchCommands(cmds, "ㅂㄹㅊ")[0]!.label).toMatch(/브랜치/);
    expect(searchCommands(cmds, "ㅂㅈㅈㅈ").map((c) => c.label)).toContain("버전 저장");
    expect(searchCommands(cmds, "버ㅈ").map((c) => c.label)).toContain("버전 저장");
    expect(searchCommands(cmds, "동기화")[0]!.key).toBe("run:sync");
    // 원래 이름으로도 찾지만 쉬운 이름보다 뒤.
    expect(searchCommands(cmds, "push").map((c) => c.key)).toContain("run:push");
    expect(searchCommands(cmds, "없는명령어")).toEqual([]);
    expect(searchCommands(cmds, "")).toEqual(cmds);
  });
});
