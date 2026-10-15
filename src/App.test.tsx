import { act, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("./lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("./lib/api")>()),
  api: {
    appStatus: vi.fn(),
    openProject: vi.fn(),
    recentProjects: vi.fn(async () => []),
    listActions: vi.fn(),
    previewAction: vi.fn(),
    executeAction: vi.fn(),
    status: vi.fn(),
    log: vi.fn(),
    diff: vi.fn(),
    homeMap: vi.fn(async () => ({ current: null, defaultBranch: null, twoLines: false, currentCommits: [], currentMore: 0, mainSince: 0, mainLast: null, upstreamOid: null, upstreamLabel: null, changedFiles: 0 })),
    branches: vi.fn(async () => ({ local: [], remote: [], defaultBranch: null, orphanParked: [], remotes: [] })),
    watch: vi.fn(async () => null),
    unwatch: vi.fn(async () => null),
  },
}));
vi.mock("./lib/dialog", () => ({ pickFolder: vi.fn() }));
vi.mock("./lib/events", () => ({
  onRepoChanged: vi.fn(async () => () => {}),
  onWatchStatus: vi.fn(async () => () => {}),
  onOpProgress: vi.fn(async () => () => {}),
}));

import { App } from "./App";
import { ApiError, api, type LogPage, type StatusView } from "./lib/api";
import { pickFolder } from "./lib/dialog";
import { useRepoStore } from "./state/repoStore";

const repo = { path: "/home/me/내 프로젝트", name: "내 프로젝트" };
const emptyStatus: StatusView = {
  branch: { name: "main", hasCommits: false, upstream: null, ahead: 0, behind: 0 },
  files: [],
};
const emptyLog: LogPage = { commits: [], hasMore: false };

beforeEach(() => {
  vi.mocked(api.appStatus).mockResolvedValue({ gitVersion: "2.45.0", gitSecurity: "ok", gitPackageVersion: null });
  vi.mocked(api.listActions).mockResolvedValue([
    { id: "init", glossaryId: "init", risk: "safe", undo: "nothing" },
  ]);
  vi.mocked(api.status).mockResolvedValue(emptyStatus);
  vi.mocked(api.log).mockResolvedValue(emptyLog);
  vi.mocked(api.watch).mockResolvedValue(null);
  vi.mocked(api.unwatch).mockResolvedValue(null);
  vi.mocked(api.recentProjects).mockResolvedValue([]);
});

afterEach(() => {
  vi.clearAllMocks();
  act(() =>
    useRepoStore.setState({
      repo: null,
      view: "home",
      selectedPath: null,
      pendingInit: null,
      startNotice: null,
    }),
  );
});

async function openFolder() {
  const user = userEvent.setup();
  render(<App />);
  const button = await screen.findByRole("button", { name: /^폴더 열기/ });
  await vi.waitFor(() => expect(button).toBeEnabled());
  await user.click(button);
  return user;
}

describe("홈에서 폴더 열기", () => {
  it("저장소를 열면 변경 사항과 버전 기록이 보인다", async () => {
    vi.mocked(pickFolder).mockResolvedValue(repo.path);
    vi.mocked(api.openProject).mockResolvedValue({ kind: "opened", repo });
    vi.mocked(api.status).mockResolvedValue({
      ...emptyStatus,
      files: [
        { path: "src/한글 파일.txt", origPath: null, change: "modified", staged: false },
        { path: "새 이름.md", origPath: "옛 이름.md", change: "renamed", staged: true },
      ],
    });
    vi.mocked(api.log).mockResolvedValue({
      commits: [
        {
          oid: "a".repeat(40),
          shortOid: "aaaaaaa",
          parents: [],
          summary: "첫 저장",
          authorName: "지원",
          authorEmail: "j@example.com",
          authorTime: 1_700_000_000,
        },
      ],
      hasMore: false,
    });

    await openFolder();

    expect(await screen.findByRole("heading", { name: "내 프로젝트" })).toBeInTheDocument();
    expect((await screen.findAllByText("src/한글 파일.txt")).length).toBeGreaterThan(0);
    expect(screen.getByText("옛 이름.md에서 이름 바꿈")).toBeInTheDocument();
    expect(api.status).toHaveBeenCalledWith(repo.path);
    // 폴더 고르기도 프로젝트 열기 한 곳을 거친다(D-032)
    expect(api.openProject).toHaveBeenCalledWith(repo.path, "picked");
    // 홈: 왼쪽 고친 파일, 오른쪽 비교. 첫 파일의 비교가 이미 열려 있다.
    expect(screen.getByText("고친 파일 2개")).toBeInTheDocument();
    await vi.waitFor(() => expect(useRepoStore.getState().selectedPath).toBe("src/한글 파일.txt"));

    // 상단 바 "버전 기록"(Ctrl+H)은 창 전체 폭 화면, Esc로 돌아온다(D-058).
    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: "버전 기록" }));
    expect(await screen.findByText("첫 저장")).toBeInTheDocument();
    expect(screen.getByText("aaaaaaa")).toBeInTheDocument();
    expect(screen.queryByText("고친 파일 2개")).not.toBeInTheDocument();
    await user.keyboard("{Escape}");
    expect(await screen.findByText("고친 파일 2개")).toBeInTheDocument();
    await user.keyboard("{Control>}h{/Control}");
    expect(await screen.findByText("첫 저장")).toBeInTheDocument();
  });

  it("저장소가 아닌 폴더는 버전 관리 시작으로 저장소를 만든다", async () => {
    vi.mocked(pickFolder).mockResolvedValue("/home/me/새 폴더");
    vi.mocked(api.openProject).mockResolvedValue({ kind: "notARepo", folder: "/home/me/새 폴더" });
    vi.mocked(api.executeAction).mockResolvedValue({
      kind: "repoCreated",
      repo: { path: "/home/me/새 폴더", name: "새 폴더" },
    });

    const user = await openFolder();

    expect(await screen.findByText("이 폴더는 아직 Git으로 관리하지 않는 폴더예요.")).toBeInTheDocument();
    const init = await screen.findByRole("button", { name: "버전 관리 시작" });
    await user.click(init);

    expect(api.executeAction).toHaveBeenCalledWith("/home/me/새 폴더", { id: "init" });
    expect(await screen.findByRole("heading", { name: "새 폴더" })).toBeInTheDocument();
    expect((await screen.findAllByText("고친 파일 없음")).length).toBeGreaterThan(0);
    await user.click(screen.getByRole("button", { name: "버전 기록" }));
    expect(await screen.findByText("아직 저장한 버전이 없어요.")).toBeInTheDocument();
  });

  it("작업 공간에서 Ctrl+O로 저장소가 아닌 폴더를 고르면 시작 화면에서 버전 관리 시작을 묻는다", async () => {
    vi.mocked(pickFolder).mockResolvedValueOnce(repo.path).mockResolvedValueOnce("/home/me/그냥 폴더");
    vi.mocked(api.openProject)
      .mockResolvedValueOnce({ kind: "opened", repo })
      .mockResolvedValueOnce({ kind: "notARepo", folder: "/home/me/그냥 폴더" });
    const user = await openFolder();
    expect(await screen.findByRole("heading", { name: "내 프로젝트" })).toBeInTheDocument();

    await user.keyboard("{Control>}o{/Control}");
    expect(await screen.findByText("이 폴더는 아직 Git으로 관리하지 않는 폴더예요.")).toBeInTheDocument();
    expect(screen.getByText("/home/me/그냥 폴더")).toBeInTheDocument();
    expect(api.openProject).toHaveBeenLastCalledWith("/home/me/그냥 폴더", "picked");
  });

  it("파일 감시를 쓸 수 없으면 안내 줄과 다시 읽기 버튼", async () => {
    vi.mocked(pickFolder).mockResolvedValue(repo.path);
    vi.mocked(api.openProject).mockResolvedValue({ kind: "opened", repo });
    vi.mocked(api.watch).mockRejectedValue(new ApiError({ code: "watchUnavailable", detail: "inotify" }));

    const user = await openFolder();
    expect(await screen.findByText("파일 변경을 자동으로 알아채지 못해요. 창으로 돌아오면 다시 읽어요.")).toBeInTheDocument();
    const calls = vi.mocked(api.status).mock.calls.length;
    await user.click(screen.getByRole("button", { name: "다시 읽기" }));
    await vi.waitFor(() => expect(vi.mocked(api.status).mock.calls.length).toBeGreaterThan(calls));
  });

  it("파일 감시가 켜져 있으면 안내 줄도 다시 읽기 버튼도 없다", async () => {
    vi.mocked(pickFolder).mockResolvedValue(repo.path);
    vi.mocked(api.openProject).mockResolvedValue({ kind: "opened", repo });
    await openFolder();
    expect(await screen.findByRole("heading", { name: "내 프로젝트" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "다시 읽기" })).not.toBeInTheDocument();
  });

  it("버전 관리 시작이 실패하면 이유와 원문을 보여준다", async () => {
    vi.mocked(pickFolder).mockResolvedValue("/잠긴 폴더");
    vi.mocked(api.openProject).mockResolvedValue({ kind: "notARepo", folder: "/잠긴 폴더" });
    vi.mocked(api.executeAction).mockRejectedValue(
      new ApiError({ code: "gitFailed", detail: "fatal: cannot mkdir .git" }),
    );

    const user = await openFolder();
    await user.click(await screen.findByRole("button", { name: "버전 관리 시작" }));

    const alert = await screen.findByRole("alert");
    expect(alert).toHaveTextContent("Git 작업이 실패했어요.");
    await user.click(within(alert).getByText("자세히 보기"));
    expect(within(alert).getByText("fatal: cannot mkdir .git")).toBeVisible();
  });

  it("폴더를 열지 못하면 쉬운 말로 알려준다", async () => {
    vi.mocked(pickFolder).mockResolvedValue("/없는 폴더");
    vi.mocked(api.openProject).mockResolvedValue({ kind: "missing" });

    await openFolder();
    expect(await screen.findByRole("alert")).toHaveTextContent("폴더를 열 수 없어요.");
  });

  it("대화상자를 취소하면 아무것도 하지 않는다", async () => {
    vi.mocked(pickFolder).mockResolvedValue(null);
    await openFolder();
    expect(api.openProject).not.toHaveBeenCalled();
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("git이 없으면 안내하고 폴더 열기를 막는다", async () => {
    vi.mocked(api.appStatus).mockRejectedValue(new ApiError({ code: "gitMissing", detail: null }));
    render(<App />);
    expect(await screen.findByRole("alert")).toHaveTextContent("Git이 설치되어 있지 않아요.");
    expect(screen.getByRole("button", { name: /^폴더 열기/ })).toBeDisabled();
  });
});

describe("앱을 켤 때", () => {
  const last = { path: repo.path, name: "내 프로젝트", lastOpenedAt: 1_700_000_000 };

  it("마지막 프로젝트를 바로 연다", async () => {
    vi.mocked(api.recentProjects).mockResolvedValue([last]);
    vi.mocked(api.openProject).mockResolvedValue({ kind: "opened", repo });
    render(<App />);
    expect(await screen.findByRole("heading", { name: "내 프로젝트" })).toBeInTheDocument();
    expect(api.openProject).toHaveBeenCalledWith(repo.path, "recent");
  });

  it("마지막 프로젝트 폴더가 사라졌으면 시작 화면에서 알린다", async () => {
    vi.mocked(api.recentProjects).mockResolvedValue([last]);
    vi.mocked(api.openProject).mockResolvedValue({ kind: "missing" });
    render(<App />);
    expect(await screen.findByRole("status")).toHaveTextContent(
      "마지막으로 연 내 프로젝트 폴더를 찾지 못했어요.",
    );
    expect(screen.getByRole("button", { name: /^폴더 열기/ })).toBeInTheDocument();
  });

  it("마지막 프로젝트가 저장소가 아니게 되었으면 그렇게 알린다", async () => {
    vi.mocked(api.recentProjects).mockResolvedValue([last]);
    vi.mocked(api.openProject).mockResolvedValue({ kind: "notARepo", folder: repo.path });
    render(<App />);
    expect(await screen.findByRole("status")).toHaveTextContent(
      "마지막으로 연 내 프로젝트 폴더는 버전 관리를 하지 않는 폴더예요.",
    );
  });

  it("git이 없으면 다시 열기를 하지 않는다", async () => {
    vi.mocked(api.appStatus).mockRejectedValue(new ApiError({ code: "gitMissing", detail: null }));
    vi.mocked(api.recentProjects).mockResolvedValue([last]);
    render(<App />);
    expect(await screen.findByRole("alert")).toBeInTheDocument();
    // 목록은 시작 화면이 보여주려고 읽지만, 프로젝트를 다시 열지는 않는다.
    expect(api.openProject).not.toHaveBeenCalled();
  });

  it("프로젝트 메뉴의 다른 프로젝트 열기로 시작 화면에 간다", async () => {
    vi.mocked(api.recentProjects).mockResolvedValue([last]);
    vi.mocked(api.openProject).mockResolvedValue({ kind: "opened", repo });
    const user = userEvent.setup();
    render(<App />);
    const menu = await screen.findByRole("button", { name: "프로젝트 메뉴: 내 프로젝트" });
    menu.focus();
    await user.keyboard("{Enter}");
    await user.click(await screen.findByRole("menuitem", { name: "다른 프로젝트 열기" }));
    expect(await screen.findByRole("button", { name: /^폴더 열기/ })).toBeInTheDocument();
    // 다시 열기를 하지 않는다
    expect(api.openProject).toHaveBeenCalledTimes(1);
  });
});
