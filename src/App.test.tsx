import { act, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("./lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("./lib/api")>()),
  api: {
    appStatus: vi.fn(),
    openFolder: vi.fn(),
    listActions: vi.fn(),
    previewAction: vi.fn(),
    executeAction: vi.fn(),
    status: vi.fn(),
    log: vi.fn(),
  },
}));
vi.mock("./lib/dialog", () => ({ pickFolder: vi.fn() }));

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
    { id: "init", glossaryId: "init", risk: "safe", inputSchema: [] },
  ]);
  vi.mocked(api.status).mockResolvedValue(emptyStatus);
  vi.mocked(api.log).mockResolvedValue(emptyLog);
});

afterEach(() => {
  vi.clearAllMocks();
  act(() => useRepoStore.setState({ repo: null }));
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
  it("저장소를 열면 바뀐 파일과 저장 기록이 보인다", async () => {
    vi.mocked(pickFolder).mockResolvedValue(repo.path);
    vi.mocked(api.openFolder).mockResolvedValue({ folder: repo.path, repo });
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
    expect(await screen.findByText("첫 저장")).toBeInTheDocument();
    expect(screen.getByText("aaaaaaa")).toBeInTheDocument();
    // 제목은 <Term>
    expect(screen.getByText("바뀐 내용")).toBeInTheDocument();
    expect(screen.getByText("저장 기록")).toBeInTheDocument();
    expect(api.status).toHaveBeenCalledWith(repo.path);
  });

  it("저장소가 아닌 폴더는 관리 시작으로 저장소를 만든다", async () => {
    vi.mocked(pickFolder).mockResolvedValue("/home/me/새 폴더");
    vi.mocked(api.openFolder).mockResolvedValue({ folder: "/home/me/새 폴더", repo: null });
    vi.mocked(api.executeAction).mockResolvedValue({
      kind: "repoCreated",
      repo: { path: "/home/me/새 폴더", name: "새 폴더" },
    });

    const user = await openFolder();

    expect(await screen.findByText("이 폴더는 아직 Git으로 관리하지 않는 폴더예요.")).toBeInTheDocument();
    const init = await screen.findByRole("button", { name: "관리 시작" });
    await user.click(init);

    expect(api.executeAction).toHaveBeenCalledWith("/home/me/새 폴더", "init");
    expect(await screen.findByRole("heading", { name: "새 폴더" })).toBeInTheDocument();
    expect(await screen.findByText("마지막 저장 이후 바뀐 파일이 없어요.")).toBeInTheDocument();
    expect(await screen.findByText("아직 저장한 기록이 없어요.")).toBeInTheDocument();
  });

  it("관리 시작이 실패하면 이유와 원문을 보여준다", async () => {
    vi.mocked(pickFolder).mockResolvedValue("/잠긴 폴더");
    vi.mocked(api.openFolder).mockResolvedValue({ folder: "/잠긴 폴더", repo: null });
    vi.mocked(api.executeAction).mockRejectedValue(
      new ApiError({ code: "gitFailed", detail: "fatal: cannot mkdir .git" }),
    );

    const user = await openFolder();
    await user.click(await screen.findByRole("button", { name: "관리 시작" }));

    const alert = await screen.findByRole("alert");
    expect(alert).toHaveTextContent("Git 작업이 실패했어요.");
    await user.click(within(alert).getByText("자세히 보기"));
    expect(within(alert).getByText("fatal: cannot mkdir .git")).toBeVisible();
  });

  it("폴더를 열지 못하면 쉬운 말로 알려준다", async () => {
    vi.mocked(pickFolder).mockResolvedValue("/없는 폴더");
    vi.mocked(api.openFolder).mockRejectedValue(
      new ApiError({ code: "folderNotFound", detail: "No such file or directory" }),
    );

    await openFolder();
    expect(await screen.findByRole("alert")).toHaveTextContent("폴더를 열 수 없어요.");
  });

  it("대화상자를 취소하면 아무것도 하지 않는다", async () => {
    vi.mocked(pickFolder).mockResolvedValue(null);
    await openFolder();
    expect(api.openFolder).not.toHaveBeenCalled();
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("git이 없으면 안내하고 폴더 열기를 막는다", async () => {
    vi.mocked(api.appStatus).mockRejectedValue(new ApiError({ code: "gitMissing", detail: null }));
    render(<App />);
    expect(await screen.findByRole("alert")).toHaveTextContent("Git이 설치되어 있지 않아요.");
    expect(screen.getByRole("button", { name: "폴더 열기" })).toBeDisabled();
  });
});
