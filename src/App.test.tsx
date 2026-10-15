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
    watch: vi.fn(async () => null),
    unwatch: vi.fn(async () => null),
  },
}));
vi.mock("./lib/dialog", () => ({ pickFolder: vi.fn() }));
vi.mock("./lib/events", () => ({
  onRepoChanged: vi.fn(async () => () => {}),
  onWatchStatus: vi.fn(async () => () => {}),
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
  vi.mocked(api.appStatus).mockResolvedValue({ gitVersion: "2.45.0" });
  vi.mocked(api.listActions).mockResolvedValue([
    { id: "init", glossaryId: "init", risk: "safe" },
  ]);
  vi.mocked(api.status).mockResolvedValue(emptyStatus);
  vi.mocked(api.log).mockResolvedValue(emptyLog);
  vi.mocked(api.watch).mockResolvedValue(null);
  vi.mocked(api.unwatch).mockResolvedValue(null);
});

afterEach(() => {
  vi.clearAllMocks();
  act(() => useRepoStore.setState({ repo: null, leftTab: "changes", selectedPath: null }));
});

async function openFolder() {
  const user = userEvent.setup();
  render(<App />);
  const button = await screen.findByRole("button", { name: "폴더 열기" });
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
    expect(await screen.findByText("src/한글 파일.txt")).toBeInTheDocument();
    expect(screen.getByText("옛 이름.md에서 이름 바꿈")).toBeInTheDocument();
    expect(api.status).toHaveBeenCalledWith(repo.path);
    // 폴더 고르기도 프로젝트 열기 한 곳을 거친다(D-032)
    expect(api.openProject).toHaveBeenCalledWith(repo.path, "picked");
    // 2단: 왼쪽 탭과 오른쪽 비교 칸, 세 번째 칸은 없다(D-025)
    const tabs = screen.getAllByRole("tab");
    expect(tabs.map((t) => t.textContent)).toEqual(["변경 사항2", "버전 기록"]);
    expect(screen.getByText("파일을 고르면 비교가 보여요.")).toBeInTheDocument();

    // 버전 기록 탭은 목록을 창 전체 폭으로
    const user = userEvent.setup();
    await user.click(screen.getByRole("tab", { name: "버전 기록" }));
    expect(await screen.findByText("첫 저장")).toBeInTheDocument();
    expect(screen.getByText("aaaaaaa")).toBeInTheDocument();
    expect(screen.queryByText("파일을 고르면 비교가 보여요.")).not.toBeInTheDocument();

    // 좌우 화살표로 탭을 옮긴다
    await user.keyboard("{ArrowLeft}");
    expect(screen.getByRole("tab", { name: /변경 사항/ })).toHaveAttribute("aria-selected", "true");
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
    expect(await screen.findByText("마지막 버전 이후 바뀐 파일이 없어요.")).toBeInTheDocument();
    await user.click(screen.getByRole("tab", { name: "버전 기록" }));
    expect(await screen.findByText("아직 저장한 버전이 없어요.")).toBeInTheDocument();
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
    expect(screen.getByRole("button", { name: "폴더 열기" })).toBeDisabled();
  });
});
