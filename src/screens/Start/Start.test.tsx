import { act, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: {
    appStatus: vi.fn(),
    recentProjects: vi.fn(),
    projectSummary: vi.fn(),
    openProject: vi.fn(),
    removeRecent: vi.fn(),
    listActions: vi.fn(async () => []),
  },
}));
vi.mock("../../lib/dialog", () => ({ pickFolder: vi.fn() }));

import { AppProviders, createQueryClient } from "../../App";
import { ApiError, api, type ProjectSummaryView } from "../../lib/api";
import { useRepoStore } from "../../state/repoStore";
import { Start } from "./Start";

const now = Math.floor(Date.now() / 1000);
const project = (name: string, minutesAgo: number) => ({
  path: `/home/me/${name}`,
  name,
  lastOpenedAt: now - minutesAgo * 60,
});
const summary = (over: Partial<ProjectSummaryView> = {}, branch: Partial<ProjectSummaryView["branch"]> = {}) => ({
  branch: { name: "main", hasCommits: true, upstream: "origin/main", ahead: 0, behind: 0, ...branch },
  changedFiles: 0,
  conflictedFiles: 0,
  lastCommit: null,
  ...over,
});

function renderStart() {
  return render(
    <AppProviders client={createQueryClient()}>
      <Start />
    </AppProviders>,
  );
}

beforeEach(() => {
  vi.mocked(api.appStatus).mockResolvedValue({ gitVersion: "2.45.0" });
  vi.mocked(api.removeRecent).mockResolvedValue(null);
});
afterEach(() => {
  vi.clearAllMocks();
  act(() => useRepoStore.setState({ repo: null, pendingInit: null, startNotice: null, selectFirstChange: false }));
});

describe("시작 화면", () => {
  it("맨 앞 프로젝트를 위쪽에, 나머지를 아래 목록에", async () => {
    vi.mocked(api.recentProjects).mockResolvedValue([
      project("my-shop", 10),
      project("blog", 60 * 24),
      project("api-server", 60 * 24 * 5),
    ]);
    vi.mocked(api.projectSummary).mockImplementation(async (repo) =>
      repo.endsWith("my-shop")
        ? summary({ changedFiles: 3 }, { name: "로그인 기능" })
        : repo.endsWith("api-server")
          ? summary({ conflictedFiles: 2, changedFiles: 2 })
          : summary({}, { ahead: 2 }),
    );
    renderStart();

    expect(await screen.findByRole("heading", { name: "my-shop" })).toBeInTheDocument();
    expect(screen.getByText("마지막으로 하던 곳 · 10분 전")).toBeInTheDocument();
    expect(
      await screen.findByText("로그인 기능 브랜치에서 파일 3개를 고치다 멈췄어요. 아직 저장한 버전이 없어요."),
    ).toBeInTheDocument();
    expect(screen.getByText("다른 프로젝트")).toBeInTheDocument();
    expect(await screen.findByText("올릴 버전 2개")).toBeInTheDocument();
    // 충돌만 빨강(D-031)
    const conflict = await screen.findByText("충돌 난 파일 2개");
    expect(conflict.className).toMatch(/danger/);
    // 주 버튼은 이어서 하기 하나
    expect(screen.getAllByRole("button", { name: "이어서 하기" })).toHaveLength(1);
  });

  it("이어서 하기와 줄 누르기는 프로젝트 열기 한 곳으로 연다", async () => {
    vi.mocked(api.recentProjects).mockResolvedValue([project("my-shop", 10), project("blog", 60)]);
    vi.mocked(api.projectSummary).mockResolvedValue(summary());
    vi.mocked(api.openProject).mockResolvedValue({ kind: "opened", repo: { path: "/home/me/my-shop", name: "my-shop" } });
    const user = userEvent.setup();
    renderStart();

    await user.click(await screen.findByRole("button", { name: "이어서 하기" }));
    expect(api.openProject).toHaveBeenCalledWith("/home/me/my-shop", "recent");
    await user.click(screen.getByRole("button", { name: /^blog/ }));
    expect(api.openProject).toHaveBeenLastCalledWith("/home/me/blog", "recent");
  });

  it("바뀐 파일 보기는 열고 나서 첫 파일을 고르게 한다", async () => {
    vi.mocked(api.recentProjects).mockResolvedValue([project("my-shop", 10)]);
    vi.mocked(api.projectSummary).mockResolvedValue(summary({ changedFiles: 3 }));
    vi.mocked(api.openProject).mockResolvedValue({ kind: "opened", repo: { path: "/home/me/my-shop", name: "my-shop" } });
    const user = userEvent.setup();
    renderStart();
    await user.click(await screen.findByRole("button", { name: "바뀐 파일 3개 보기" }));
    expect(useRepoStore.getState().selectFirstChange).toBe(true);
    expect(api.openProject).toHaveBeenCalledWith("/home/me/my-shop", "recent");
  });

  it("맨 앞 폴더가 사라졌으면 다음 프로젝트를 위쪽에, 사라진 줄은 목록에서 뺄 수 있다", async () => {
    vi.mocked(api.recentProjects).mockResolvedValue([project("사라진 폴더", 10), project("blog", 60)]);
    vi.mocked(api.projectSummary).mockImplementation(async (repo) => {
      if (repo.endsWith("사라진 폴더")) throw new ApiError({ code: "folderNotFound", detail: null });
      return summary();
    });
    const user = userEvent.setup();
    renderStart();

    expect(await screen.findByRole("heading", { name: "blog" })).toBeInTheDocument();
    const row = (await screen.findByText("폴더를 찾을 수 없어요")).closest("li")!;
    expect(within(row).getByText("사라진 폴더").className).toMatch(/muted/);
    await user.click(within(row).getByRole("button", { name: "목록에서 빼기" }));
    expect(api.removeRecent).toHaveBeenCalledWith("/home/me/사라진 폴더");
  });

  it("연 프로젝트가 없으면 빈 상태와 찾은 Git 버전", async () => {
    vi.mocked(api.recentProjects).mockResolvedValue([]);
    renderStart();
    expect(await screen.findByText("아직 연 프로젝트가 없어요. 오른쪽 위의 폴더 열기로 시작해요.")).toBeInTheDocument();
    expect(await screen.findByText("✓ Git을 찾았어요 · 버전 2.45.0")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "이어서 하기" })).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: /^폴더 열기/ })).toHaveTextContent("Ctrl+O");
  });
});
