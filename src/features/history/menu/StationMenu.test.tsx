import { QueryClient } from "@tanstack/react-query";
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("../../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../../lib/api")>()),
  api: {
    log: vi.fn(),
    logStats: vi.fn(async () => []),
    status: vi.fn(async () => ({ branch: { name: "main", hasCommits: true, upstream: null, ahead: 0, behind: 0 }, files: [] })),
  },
}));

import { AppProviders, createQueryClient } from "../../../App";
import { api, type LogQuery } from "../../../lib/api";
import actionMeta from "../../../lib/action-meta.json";
import { ko } from "../../../messages/ko";
import { useHistoryStore } from "../../../state/historyStore";
import { useRepoStore } from "../../../state/repoStore";
import { History } from "../../../screens/History/History";
import { logPage } from "../../../test/logPage";
import { useToastStore } from "../../../ui/toastStore";
import { commitNode } from "../model";
import { headAncestors, stationInfo } from "./info";
import { STATION_ITEMS, type MenuCtx, type MenuEnv, type StationMenuItem } from "./items";
import { StationMenu } from "./StationMenu";

const oid = (i: number) => `${i}`.padStart(40, "0");

function serve() {
  vi.mocked(api.log).mockImplementation(async (_r: string, q: LogQuery) =>
    logPage({
      commits: [0, 1, 2].map((i) => ({
        oid: oid(i),
        shortOid: oid(i).slice(0, 7),
        parents: i < 2 ? [oid(i + 1)] : [],
        summary: `버전 ${i}`,
        authorName: "지원",
        authorEmail: "j@e",
        authorTime: 1_700_000_000,
      })),
      head: q.tips ? null : oid(0),
      tips: ["t"],
    }),
  );
}

function showHistory() {
  render(
    <AppProviders client={createQueryClient()}>
      <History repo="/repo" onDone={() => {}} />
    </AppProviders>,
  );
}

afterEach(() => {
  vi.clearAllMocks();
  vi.unstubAllGlobals();
  act(() => {
    useHistoryStore.setState({ selectedId: null, zoom: "all", scope: "all", expanded: new Set() });
    useToastStore.setState({ toast: null });
    useRepoStore.setState({ view: "history" });
  });
});

describe("역 메뉴", () => {
  it("Enter로 열고, 지금 위치에는 버전 번호 복사와 되돌리기가 있고 더 보기는 없다", async () => {
    serve();
    showHistory();
    const list = await screen.findByRole("listbox");
    list.focus();
    fireEvent.keyDown(list, { key: "ArrowDown" });
    fireEvent.keyDown(list, { key: "Enter" });
    const menu = await screen.findByRole("menu", { name: "버전 메뉴" });
    expect(menu).toHaveTextContent("버전 번호 복사");
    expect(menu).toHaveTextContent("되돌리기");
    expect(screen.queryByText("더 보기")).not.toBeInTheDocument();
  });

  it("Esc는 메뉴만 닫고 목록에 포커스를 돌려준다. 한 번 더 누르면 홈", async () => {
    serve();
    showHistory();
    const list = await screen.findByRole("listbox");
    list.focus();
    fireEvent.keyDown(list, { key: "ArrowDown" });
    fireEvent.keyDown(list, { key: "Enter" });
    await screen.findByRole("menu");
    await userEvent.keyboard("{Escape}");
    await waitFor(() => expect(screen.queryByRole("menu")).not.toBeInTheDocument());
    expect(useRepoStore.getState().view).toBe("history");
    expect(list).toHaveFocus();
    await userEvent.keyboard("{Escape}");
    expect(useRepoStore.getState().view).toBe("home");
  });

  it("버전 번호 복사: 클립보드에 해시를 넣고 토스트", async () => {
    serve();
    const writeText = vi.fn(async () => {});
    vi.stubGlobal("navigator", { ...navigator, clipboard: { writeText } });
    showHistory();
    const list = await screen.findByRole("listbox");
    list.focus();
    fireEvent.keyDown(list, { key: "ArrowDown" });
    fireEvent.keyDown(list, { key: "Enter" });
    // user-event는 클립보드를 제 것으로 바꾸므로 눌림만 보낸다.
    fireEvent.click(await screen.findByRole("menuitem", { name: "버전 번호 복사" }));
    expect(writeText).toHaveBeenCalledWith(oid(0));
    await waitFor(() => expect(useToastStore.getState().toast?.text).toBe("복사했어요"));
  });

  it("클립보드가 없으면 옛 방법으로, 그것도 안 되면 알린다", async () => {
    serve();
    vi.stubGlobal("navigator", { ...navigator, clipboard: undefined });
    const exec = vi.fn(() => false);
    document.execCommand = exec;
    showHistory();
    const list = await screen.findByRole("listbox");
    list.focus();
    fireEvent.keyDown(list, { key: "ArrowDown" });
    fireEvent.keyDown(list, { key: "Enter" });
    // user-event는 클립보드를 제 것으로 바꾸므로 눌림만 보낸다.
    fireEvent.click(await screen.findByRole("menuitem", { name: "버전 번호 복사" }));
    expect(exec).toHaveBeenCalledWith("copy");
    await waitFor(() => expect(useToastStore.getState().toast?.text).toBe("복사하지 못했어요"));
  });

  it("메뉴 안의 누름·키는 목록의 선택을 바꾸지 않는다(포털 이벤트)", async () => {
    serve();
    const writeText = vi.fn(async () => {});
    vi.stubGlobal("navigator", { ...navigator, clipboard: { writeText } });
    showHistory();
    const list = await screen.findByRole("listbox");
    list.getBoundingClientRect = () => ({ top: 0, left: 0, right: 800, bottom: 600, width: 800, height: 600, x: 0, y: 0, toJSON: () => ({}) });
    Object.defineProperty(list, "clientWidth", { value: 780, configurable: true });
    list.focus();
    fireEvent.keyDown(list, { key: "ArrowDown" });
    fireEvent.keyDown(list, { key: "Enter" });
    const item = await screen.findByRole("menuitem", { name: "버전 번호 복사" });
    // 셋째 행 높이에서 메뉴 항목을 누르고, 메뉴 안에서 ↓를 눌러도 선택은 첫 행 그대로다.
    fireEvent.pointerDown(item, { button: 0, clientX: 300, clientY: 120 });
    fireEvent.keyDown(item, { key: "ArrowDown" });
    expect(list.getAttribute("aria-activedescendant")).toBe("route-row-0");
    fireEvent.click(item);
    expect(writeText).toHaveBeenCalledWith(oid(0));
  });

  it("메뉴 항목에서 Enter로 고르면 메뉴가 닫힌 채로 남는다", async () => {
    serve();
    vi.stubGlobal("navigator", { ...navigator, clipboard: { writeText: vi.fn(async () => {}) } });
    showHistory();
    const list = await screen.findByRole("listbox");
    list.focus();
    fireEvent.keyDown(list, { key: "ArrowDown" });
    fireEvent.keyDown(list, { key: "Enter" });
    const item = await screen.findByRole("menuitem", { name: "버전 번호 복사" });
    item.focus();
    fireEvent.keyDown(item, { key: "Enter" });
    await waitFor(() => expect(screen.queryByRole("menu")).not.toBeInTheDocument());
  });

  it("오른쪽 클릭은 웹뷰 기본 메뉴를 막고 그 역을 골라 연다", async () => {
    serve();
    showHistory();
    const list = await screen.findByRole("listbox");
    list.getBoundingClientRect = () => ({ top: 0, left: 0, right: 800, bottom: 600, width: 800, height: 600, x: 0, y: 0, toJSON: () => ({}) });
    Object.defineProperty(list, "clientWidth", { value: 780, configurable: true });
    // 셋째 행(행 높이 48, y=130)을 오른쪽 클릭.
    const allowed = fireEvent.contextMenu(list, { clientX: 300, clientY: 130 });
    expect(allowed).toBe(false);
    await screen.findByRole("menu");
    expect(useHistoryStore.getState().selectedId).toBe(oid(2));
    // 열린 채로 같은 역을 다시 오른쪽 클릭해도 그대로 열려 있다.
    fireEvent.contextMenu(list, { clientX: 300, clientY: 130 });
    expect(screen.getByRole("menu")).toBeInTheDocument();
  });

  it("접힌 구간을 펼치면 그 안의 가장 새 버전을 고른다", async () => {
    serve();
    act(() => useHistoryStore.setState({ zoom: "key" }));
    showHistory();
    const list = await screen.findByRole("listbox");
    list.focus();
    fireEvent.keyDown(list, { key: "ArrowDown" });
    fireEvent.keyDown(list, { key: "ArrowDown" });
    fireEvent.keyDown(list, { key: "Enter" });
    await waitFor(() => expect(useHistoryStore.getState().selectedId).toBe(oid(1)));
  });

  it("같은 역을 다시 누르면 닫는다", async () => {
    serve();
    showHistory();
    const list = await screen.findByRole("listbox");
    list.getBoundingClientRect = () => ({ top: 0, left: 0, right: 800, bottom: 600, width: 800, height: 600, x: 0, y: 0, toJSON: () => ({}) });
    Object.defineProperty(list, "clientWidth", { value: 780, configurable: true });
    fireEvent.pointerDown(list, { button: 0, clientX: 300, clientY: 70 });
    await screen.findByRole("menu");
    fireEvent.pointerDown(list, { button: 0, clientX: 300, clientY: 70 });
    await waitFor(() => expect(screen.queryByRole("menu")).not.toBeInTheDocument());
  });
});

describe("역 메뉴 항목 표", () => {
  const info = stationInfo(commitNode(oid(1), [oid(2)], { summary: "a" }), new Set(), new Set())!;
  const env: MenuEnv = {
    repo: "/r",
    currentBranch: "main",
    changedFiles: 0,
    stagedFiles: 0,
    operation: "none",
    upstream: null,
    defaultBranch: "main",
    zoom: "all",
  };
  const ctx: MenuCtx = {
    repo: "/r",
    toast: () => {},
    queryClient: new QueryClient(),
    onDone: () => {},
    ask: () => {},
    openActivity: () => {},
    m: ko,
  };
  const item = (id: string, group: "main" | "more", availability: StationMenuItem["availability"]): StationMenuItem => ({
    id,
    group,
    label: () => id,
    availability,
    run: () => {},
  });

  it("보임·숨김·흐림(까닭 툴팁)과 더 보기", async () => {
    render(
      <AppProviders client={createQueryClient()}>
        <StationMenu
          open
          onOpenChange={() => {}}
          info={info}
          env={env}
          ctx={ctx}
          anchor={{ x: 0, y: 0 }}
          returnFocus={() => {}}
          isInside={() => false}
          items={[
            item("보임", "main", () => "show"),
            item("숨김", "main", () => "hide"),
            item("흐림", "main", () => ({ disabled: () => "고친 파일이 있어요" })),
            item("깊이", "more", () => "show"),
          ]}
        />
      </AppProviders>,
    );
    expect(await screen.findByRole("menuitem", { name: "보임" })).toBeInTheDocument();
    expect(screen.queryByText("숨김")).not.toBeInTheDocument();
    const dim = screen.getByRole("menuitem", { name: "흐림" });
    // 흐린 항목도 키보드로 닿고(Radix disabled가 아님) 까닭이 툴팁으로 뜬다.
    expect(dim).toHaveAttribute("aria-disabled", "true");
    expect(dim).not.toHaveAttribute("data-disabled");
    act(() => dim.focus());
    expect((await screen.findByRole("tooltip")).textContent).toContain("고친 파일이 있어요");
    expect(screen.getByText("더 보기")).toBeInTheDocument();
  });

  it("흐린 항목은 골라도 실행하지 않고 메뉴도 닫지 않는다", async () => {
    const run = vi.fn();
    const onOpenChange = vi.fn();
    render(
      <AppProviders client={createQueryClient()}>
        <StationMenu
          open
          onOpenChange={onOpenChange}
          info={info}
          env={env}
          ctx={ctx}
          anchor={{ x: 0, y: 0 }}
          returnFocus={() => {}}
          isInside={() => false}
          items={[{ ...item("흐림", "main", () => ({ disabled: () => "까닭" })), run }]}
        />
      </AppProviders>,
    );
    fireEvent.click(await screen.findByRole("menuitem", { name: "흐림" }));
    expect(run).not.toHaveBeenCalled();
    expect(onOpenChange).not.toHaveBeenCalledWith(false);
    expect(screen.getByRole("menu")).toBeInTheDocument();
  });

  it("액션 항목은 action-meta에 있는 id여야 한다", () => {
    const ids = new Set(actionMeta.map((a) => a.id));
    for (const i of STATION_ITEMS) if (i.actionId) expect(ids.has(i.actionId), i.id).toBe(true);
    expect(item("x", "main", () => "show").availability(info, env)).toBe("show");
  });

  it("역 정보: 합치기·첫 버전·지금 위치·조상·이름들", () => {
    const nodes = [
      commitNode("h", ["m"], { isHead: true, branches: ["main", "기능"] }),
      commitNode("m", ["a", "s"], { tags: ["v1"], unpushed: true }),
      commitNode("s", ["a"]),
      commitNode("a", []),
    ];
    const anc = headAncestors(nodes);
    const head = stationInfo(nodes[0]!, new Set(["h", "m", "a"]), anc)!;
    expect(head).toMatchObject({ isHead: true, onHeadLine: true, branches: ["main", "기능"] });
    const merge = stationInfo(nodes[1]!, new Set(["h", "m", "a"]), anc)!;
    expect(merge).toMatchObject({ isMerge: true, tags: ["v1"], unpushed: true });
    const side = stationInfo(nodes[2]!, new Set(["h", "m", "a"]), anc)!;
    expect(side).toMatchObject({ onHeadLine: false, isAncestorOfHead: true });
    expect(stationInfo(nodes[3]!, new Set(), anc)!.isRoot).toBe(true);
    expect(stationInfo({ ...nodes[3]!, kind: "fold" }, new Set(), anc)).toBeNull();
  });
});
