import { act, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: {
    listActions: vi.fn(),
    status: vi.fn(),
    branches: vi.fn(),
    recentProjects: vi.fn(),
    executeAction: vi.fn(),
  },
}));

import { AppProviders, createQueryClient } from "../../App";
import { api, type ActionMetaView, type BranchListView, type StatusView } from "../../lib/api";
import meta from "../../lib/action-meta.json";
import { useRepoStore } from "../../state/repoStore";
import { useBranchListStore } from "../branches/branchListStore";
import { Palette } from "./Palette";
import { usePaletteStore } from "./paletteStore";

const status = (files: number): StatusView => ({
  branch: { name: "main", hasCommits: true, upstream: "origin/main", ahead: 0, behind: 0 },
  files: Array.from({ length: files }, (_, i) => ({ path: `f${i}`, origPath: null, change: "modified", staged: false })),
});
const row = (name: string, remote: string | null = null) => ({
  name,
  remote,
  isHead: name === "main",
  isDefault: name === "main",
  upstream: null,
  ahead: 0,
  behind: 0,
  gone: false,
  lastAuthor: "민지",
  lastAt: 1,
  parkedCount: 0,
  parkBlocked: false,
});
const branches: BranchListView = {
  local: [row("main"), row("디자인-수정")],
  remote: [],
  defaultBranch: "main",
  orphanParked: [],
  remotes: ["origin"],
};

function show({ repo = "/repo" as string | null, files = 2, onDone = vi.fn() } = {}) {
  vi.mocked(api.listActions).mockResolvedValue(meta as ActionMetaView[]);
  vi.mocked(api.status).mockResolvedValue(status(files));
  vi.mocked(api.branches).mockResolvedValue(branches);
  vi.mocked(api.recentProjects).mockResolvedValue([{ path: "/home/me/blog", name: "blog", lastOpenedAt: 1 , pinned: false, pinnedBranches: []}]);
  render(
    <AppProviders client={createQueryClient()}>
      <Palette repo={repo} onDone={onDone} />
    </AppProviders>,
  );
  return onDone;
}

const openPalette = () => fireEvent.keyDown(window, { code: "KeyK", key: "k", ctrlKey: true });
const options = () => within(screen.getByRole("listbox")).getAllByRole("option");

afterEach(() => {
  vi.clearAllMocks();
  act(() => {
    usePaletteStore.setState({ open: false });
    useBranchListStore.setState({ open: false, asking: null });
    useRepoStore.setState({ view: "home" });
  });
});

describe("명령 팔레트(D-170, D-199)", () => {
  it("Ctrl+K로 열고 찾기 칸에 포커스, 안에서 Ctrl+K로 닫는다", async () => {
    show();
    openPalette();
    const box = await screen.findByRole("combobox", { name: "명령이나 브랜치 이름" });
    await waitFor(() => expect(box).toHaveFocus());
    fireEvent.keyDown(box, { code: "KeyK", key: "k", ctrlKey: true });
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });

  it("초성으로 찾고 Enter로 바로 한다", async () => {
    const onDone = show();
    vi.mocked(api.executeAction).mockResolvedValue({ kind: "stashed" });
    openPalette();
    const box = await screen.findByRole("combobox");
    await waitFor(() => expect(options().some((o) => o.textContent?.includes("임시 보관"))).toBe(true));
    await userEvent.type(box, "ㅂㄹㅊ");
    expect(options()[0]).toHaveTextContent("브랜치");
    await userEvent.clear(box);
    await userEvent.type(box, "임시 보관");
    expect(box).toHaveAttribute("aria-activedescendant", options()[0]!.id);
    await userEvent.keyboard("{Enter}");
    expect(api.executeAction).toHaveBeenCalledWith("/repo", { id: "stash" });
    await vi.waitFor(() => expect(onDone).toHaveBeenCalledWith("보관했어요"));
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });

  it("화면 열기 항목과 ↓ 이동", async () => {
    show();
    openPalette();
    const box = await screen.findByRole("combobox");
    await userEvent.type(box, "버전 기록");
    await waitFor(() => expect(options()[0]).toHaveTextContent("버전 기록"));
    await userEvent.keyboard("{Enter}");
    expect(useRepoStore.getState().view).toBe("history");
  });

  it("확인이 필요한 액션은 흐리고 까닭, 눌러도 하지 않는다", async () => {
    show();
    openPalette();
    const box = await screen.findByRole("combobox");
    await userEvent.type(box, "검토 요청 합치기");
    const dim = await waitFor(() => options().find((o) => o.getAttribute("aria-disabled") === "true")!);
    expect(dim).toHaveTextContent("검토 요청 화면에서 해요");
    await userEvent.click(dim);
    expect(api.executeAction).not.toHaveBeenCalled();
    expect(screen.getByRole("dialog")).toBeInTheDocument();
  });

  it("한글 조합 중 Enter는 무시한다", async () => {
    show();
    openPalette();
    const box = await screen.findByRole("combobox");
    await waitFor(() => expect(options().length).toBeGreaterThan(0));
    fireEvent.keyDown(box, { key: "Enter", keyCode: 229 });
    expect(screen.getByRole("dialog")).toBeInTheDocument();
  });

  it("고친 파일이 있으면 브랜치 바꾸기는 두고 갈지 묻는 대화상자로", async () => {
    show({ files: 3 });
    openPalette();
    const box = await screen.findByRole("combobox");
    await userEvent.type(box, "디자인");
    await waitFor(() => expect(options()[0]).toHaveTextContent("디자인-수정"));
    await userEvent.keyboard("{Enter}");
    expect(useBranchListStore.getState().asking).toEqual({ target: { branch: "디자인-수정", remote: null }, files: 3 });
    expect(api.executeAction).not.toHaveBeenCalled();
  });

  it("시작 화면에서는 폴더 열기·복제·설정·단축키 표·최근 프로젝트만", async () => {
    show({ repo: null });
    openPalette();
    await waitFor(() => expect(options().map((o) => o.querySelector("span")?.textContent)).toEqual(["폴더 열기", "복제", "설정", "단축키 보기", "blog"]));
    expect(api.status).not.toHaveBeenCalled();
  });

  it("찾는 것이 없으면 빈 줄 문구", async () => {
    show();
    openPalette();
    await userEvent.type(await screen.findByRole("combobox"), "없는명령어");
    expect(screen.getByText("찾는 명령이 없어요")).toBeInTheDocument();
  });
});
