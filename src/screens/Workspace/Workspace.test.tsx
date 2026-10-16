import { act, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { FileDiffView, HomeMapView, StatusView } from "../../lib/api";

const fixtures = vi.hoisted(() => {
  const c = (n: number) => ({ oid: `${n}`.padStart(40, "0"), shortOid: `${n}`, summary: `버전 ${n}` });
  const status: StatusView = {
    branch: { name: "main", hasCommits: true, upstream: null, ahead: 0, behind: 0 },
    files: [
      { path: "src/app.ts", origPath: null, change: "modified", staged: false },
      { path: "README.md", origPath: null, change: "modified", staged: false },
    ],
  };
  const homeMap: HomeMapView = {
    current: "main",
    defaultBranch: "main",
    twoLines: false,
    currentCommits: [c(1), c(2)],
    currentMore: 3,
    mainSince: 0,
    mainLast: null,
    upstreamOid: null,
    upstreamLabel: null,
    changedFiles: 2,
  };
  const ctx = (n: number) => ({ kind: "context" as const, text: `줄 ${n}`, oldLine: n, newLine: n, noNewlineAtEnd: false });
  const diff: FileDiffView = {
    binary: false,
    truncated: false,
    link: null,
    hunks: [{ section: "", oldStart: 20, oldLen: 2, newStart: 20, newLen: 2, lines: [ctx(20), ctx(21)] }],
  };
  return { status, homeMap, diff };
});

// 화면이 부르는 조회만 답하고 나머지는 실패로 둔다(오류 알림은 Tab 순서와 상관없다).
vi.mock("../../lib/api", async (importOriginal) => {
  const real = await importOriginal<typeof import("../../lib/api")>();
  const known: Record<string, () => Promise<unknown>> = {
    status: async () => fixtures.status,
    homeMap: async () => fixtures.homeMap,
    diff: async () => fixtures.diff,
    branches: async () => ({ local: [], remote: [], defaultBranch: "main", orphanParked: [], remotes: [] }),
    conflictState: async () => ({ active: false, otherOperation: false, files: [], ours: null, theirs: null }),
    watch: async () => null,
    unwatch: async () => null,
  };
  const api = new Proxy({}, { get: (_, key: string) => known[key] ?? (() => Promise.reject(new Error(`unmocked ${key}`))) });
  return { ...real, api };
});
vi.mock("../../lib/events", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/events")>()),
  onRepoChanged: vi.fn(async () => () => {}),
  onWatchStatus: vi.fn(async () => () => {}),
  onOpProgress: vi.fn(async () => () => {}),
}));

import { AppProviders, createQueryClient } from "../../App";
import { useRepoStore } from "../../state/repoStore";
import { Workspace } from "./Workspace";

afterEach(() => {
  act(() => useRepoStore.setState({ view: "home", selectedPath: null }));
});

describe("Workspace Tab 순서(C14)", () => {
  it("상단 바 → 노선도 → 이름 칸 → 파일 목록 → 나눔선 → 비교 칸", async () => {
    const user = userEvent.setup();
    render(
      <AppProviders client={createQueryClient()}>
        <Workspace repo={{ path: "/home/me/todo", name: "todo" }} />
      </AppProviders>,
    );
    const earlier = await screen.findByRole("button", { name: "이전 3개" });
    await screen.findByText("줄 20");

    const regions: [string, Element][] = [
      ["상단 바", document.querySelector("header")!],
      ["노선도", earlier.parentElement!],
      ["이름 칸", document.querySelector("form")!],
      ["파일 목록", document.getElementById("changed-files")!],
      ["나눔선", screen.getByRole("separator")],
      ["비교 칸", screen.getByRole("region", { name: "비교 보기" })],
    ];
    const seen: string[] = [];
    for (let i = 0; i < 60; i++) {
      await user.tab();
      const el = document.activeElement;
      if (!el || el === document.body) break;
      const region = regions.find(([, root]) => root.contains(el))?.[0] ?? `밖: ${el.tagName} ${el.textContent?.slice(0, 30) ?? ""}`;
      if (seen.at(-1) !== region) seen.push(region);
      if (seen.length > regions.length + 2) break;
    }
    expect(seen).toEqual(regions.map(([name]) => name));
    // 양수 tabIndex는 문서 순서를 깨므로 쓰지 않는다.
    expect(document.querySelector("[tabindex]:not([tabindex='0']):not([tabindex='-1'])")).toBeNull();
  });
});
