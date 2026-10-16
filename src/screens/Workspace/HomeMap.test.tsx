import { act, fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { HomeMapView, MapCommit } from "../../lib/api";

const homeMap = vi.fn<() => Promise<HomeMapView>>();
vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: {
    homeMap: () => homeMap(),
    mergeHint: vi.fn(async () => ({ kind: "none", branch: null, behind: 0, ahead: 0, overlap: [], overlapCount: 0 })),
  },
}));

import { AppProviders, createQueryClient } from "../../App";
import { useRepoStore } from "../../state/repoStore";
import { useUiStore } from "../../state/uiStore";
import { HomeMap } from "./HomeMap";

const c = (n: number, summary = `버전 ${n}`): MapCommit => ({ oid: `${n}`.padStart(40, "0"), shortOid: `${n}`, summary });
const view = (over: Partial<HomeMapView> = {}): HomeMapView => ({
  current: "main",
  defaultBranch: "main",
  twoLines: false,
  currentCommits: [c(1), c(2), c(3)],
  currentMore: 0,
  mainSince: 0,
  mainLast: null,
  upstreamOid: null,
  upstreamLabel: null,
  changedFiles: 0,
  ...over,
});

function show() {
  render(
    <AppProviders client={createQueryClient()}>
      <HomeMap repo="/repo" onDone={vi.fn()} />
    </AppProviders>,
  );
}

afterEach(() => {
  act(() => useRepoStore.setState({ view: "home" }));
});

describe("홈 노선도 연결(D-141, D-155)", () => {
  it("접힌 버전이 있으면 이전 N개 단추, 누르면 버전 기록", async () => {
    homeMap.mockResolvedValue(view({ currentMore: 3 }));
    show();
    const earlier = await screen.findByRole("button", { name: "이전 3개" });
    fireEvent.click(earlier);
    expect(useRepoStore.getState().view).toBe("history");
    // SVG 글자로는 그리지 않는다(단추와 겹치지 않게).
    expect(document.querySelector("svg text")?.textContent).not.toContain("이전");
  });

  it("접힌 버전이 없으면 단추가 없다", async () => {
    homeMap.mockResolvedValue(view());
    show();
    await screen.findAllByRole("img", { name: "버전 1" });
    expect(screen.queryByRole("button", { name: /이전/ })).toBeNull();
  });

  it("역에 포커스하면 전체 이름 툴팁, 라벨은 잘려도 툴팁은 전체", async () => {
    const long = "로그인 화면을 처음부터 다시 만들고 버튼 문구를 모두 정리";
    homeMap.mockResolvedValue(view({ currentCommits: [c(1, long), c(2)] }));
    show();
    const station = await screen.findByRole("img", { name: long });
    await userEvent.tab();
    expect(station).toHaveFocus();
    expect(await screen.findByRole("tooltip")).toHaveTextContent(long);
  });

  it("Tab 순서는 이전 N개 → 역들(왼쪽부터)", async () => {
    homeMap.mockResolvedValue(view({ currentMore: 2 }));
    show();
    const earlier = await screen.findByRole("button", { name: "이전 2개" });
    await userEvent.tab();
    expect(earlier).toHaveFocus();
    await userEvent.tab();
    expect(document.activeElement?.getAttribute("aria-label")).toBe("버전 1");
  });

  it("배율 1.5면 단추 자리도 1.5배", async () => {
    homeMap.mockResolvedValue(view({ currentMore: 2 }));
    act(() => useUiStore.setState({ uiScale: 1 }));
    show();
    const at1 = Number.parseFloat((await screen.findByRole("button", { name: "이전 2개" })).style.left);
    act(() => useUiStore.setState({ uiScale: 1.5 }));
    const at15 = Number.parseFloat(screen.getByRole("button", { name: "이전 2개" }).style.left);
    expect(at15).toBeCloseTo(at1 * 1.5);
  });
});

describe("홈 노선도 폭", () => {
  it("자료가 온 뒤 생긴 칸의 폭을 잰다", async () => {
    const observed: Element[] = [];
    const original = globalThis.ResizeObserver;
    globalThis.ResizeObserver = class {
      constructor(private readonly cb: ResizeObserverCallback) {}
      observe(el: Element) {
        observed.push(el);
        this.cb([{ contentRect: { width: 300 } } as ResizeObserverEntry], this as unknown as ResizeObserver);
      }
      unobserve() {}
      disconnect() {}
    } as unknown as typeof ResizeObserver;
    try {
      homeMap.mockResolvedValue(view({ twoLines: true, current: "기능", mainSince: 1, mainLast: c(9) }));
      show();
      await screen.findAllByRole("img", { name: "버전 1" });
      expect(observed).toHaveLength(1);
      // 좁으면 지금 줄만(기본 브랜치 줄 없음).
      expect(document.querySelector("svg")?.getAttribute("width")).toBe("320");
    } finally {
      globalThis.ResizeObserver = original;
    }
  });
});
