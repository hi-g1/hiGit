import { act, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: {
    appStatus: vi.fn(),
    openGitDownload: vi.fn(),
    recentProjects: vi.fn(),
    projectSummary: vi.fn(),
    openProject: vi.fn(),
    removeRecent: vi.fn(),
    listActions: vi.fn(async () => []),
    routePreview: vi.fn(async () => ({
      current: "로그인-기능",
      defaultBranch: "main",
      twoLines: true,
      currentCommits: [{ oid: "1".repeat(40), shortOid: "1111111", summary: "로그인 화면" }],
      currentMore: 0,
      mainSince: 1,
      mainLast: { oid: "2".repeat(40), shortOid: "2222222", summary: "오타 수정" },
      upstreamOid: null,
      upstreamLabel: null,
      changedFiles: 0,
    })),
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
  pinned: false,
  pinnedBranches: [] as string[],
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
  vi.mocked(api.appStatus).mockResolvedValue({ gitVersion: "2.45.0", gitSecurity: "ok", gitPackageVersion: null, gitPath: "/usr/bin/git" });
  vi.mocked(api.removeRecent).mockResolvedValue(null);
});
afterEach(() => {
  vi.clearAllMocks();
  act(() => useRepoStore.setState({ repo: null, pendingInit: null, startNotice: null, selectFirstChange: false }));
});

describe("시작 화면", () => {
  it("상단 바 맨 앞에 로고", async () => {
    vi.mocked(api.recentProjects).mockResolvedValue([]);
    renderStart();
    const bar = screen.getByRole("banner");
    expect(bar.firstElementChild?.querySelector('img[alt="hiGit"]')).not.toBeNull();
  });

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
    // 이름은 Truncate 안의 bdi다. 흐린 색은 그 바깥 줄에 있다.
    expect(within(row).getByText("사라진 폴더").parentElement?.className).toMatch(/muted/);
    await user.click(within(row).getByRole("button", { name: "목록에서 빼기" }));
    expect(api.removeRecent).toHaveBeenCalledWith("/home/me/사라진 폴더");
  });

  it("마지막 프로젝트의 큰 노선도: 상태 요약이 성공한 뒤에만, 글자에는 브랜치 색 없음", async () => {
    vi.mocked(api.recentProjects).mockResolvedValue([project("my-shop", 10)]);
    vi.mocked(api.projectSummary).mockResolvedValue(summary());
    renderStart();
    const map = await screen.findByRole("img", { name: "버전 기록 · 로그인-기능 · 버전 1개" });
    expect(api.routePreview).toHaveBeenCalledWith("/home/me/my-shop", 12);
    expect(map.textContent).toContain("로그인 화면");
    for (const t of map.querySelectorAll("text")) expect(t.getAttribute("fill") ?? "").not.toMatch(/graph/);
    expect(screen.getAllByRole("button", { name: "이어서 하기" })).toHaveLength(1);
  });

  it("사라진 폴더·신뢰 필요·저장 없는 프로젝트는 노선도를 부르지 않는다", async () => {
    vi.mocked(api.recentProjects).mockResolvedValue([project("믿음 필요", 10), project("빈 것", 20)]);
    vi.mocked(api.projectSummary).mockImplementation(async (repo) => {
      if (repo.endsWith("믿음 필요")) throw new ApiError({ code: "needsTrust", detail: null });
      return summary({}, { hasCommits: false });
    });
    renderStart();
    expect(await screen.findByRole("heading", { name: "믿음 필요" })).toBeInTheDocument();
    await screen.findByText("확인 필요");
    expect(api.routePreview).not.toHaveBeenCalled();
  });

  it("작은 노선도는 줄이 화면에 들어올 때 부른다", async () => {
    const watchers: ((entries: { isIntersecting: boolean }[]) => void)[] = [];
    vi.stubGlobal(
      "IntersectionObserver",
      class {
        constructor(cb: (entries: { isIntersecting: boolean }[]) => void) {
          watchers.push(cb);
        }
        observe() {}
        disconnect() {}
      },
    );
    vi.mocked(api.recentProjects).mockResolvedValue([project("my-shop", 10), project("blog", 60)]);
    vi.mocked(api.projectSummary).mockResolvedValue(summary());
    renderStart();
    expect(await screen.findByText("blog")).toBeInTheDocument();
    await screen.findByRole("img", { name: /버전 기록/ });
    expect(api.routePreview).not.toHaveBeenCalledWith("/home/me/blog", 6);
    act(() => watchers.forEach((w) => w([{ isIntersecting: true }])));
    await vi.waitFor(() => expect(api.routePreview).toHaveBeenCalledWith("/home/me/blog", 6));
    vi.unstubAllGlobals();
  });

  it("연 프로젝트가 없으면 빈 상태와 찾은 Git 버전", async () => {
    vi.mocked(api.recentProjects).mockResolvedValue([]);
    renderStart();
    expect(await screen.findByText("아직 연 프로젝트가 없어요. 오른쪽 위의 폴더 열기나 복제로 시작해요.")).toBeInTheDocument();
    expect(await screen.findByText("✓ Git을 찾았어요 · 버전 2.45.0")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "이어서 하기" })).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: /^폴더 열기/ })).toHaveTextContent("Ctrl+O");
    const clone = screen.getByRole("button", { name: /^복제/ });
    expect(clone).toHaveTextContent("Ctrl+Shift+O");
    expect(clone).toHaveAttribute("aria-keyshortcuts", "Control+Shift+O");
    expect(screen.getByRole("button", { name: "설정" })).toBeInTheDocument();
  });

  it("Git이 보안 수정 기준보다 낮으면 업데이트 안내와 내려받기", async () => {
    vi.mocked(api.recentProjects).mockResolvedValue([]);
    vi.mocked(api.appStatus).mockResolvedValue({ gitVersion: "2.40.0", gitSecurity: "outdated", gitPackageVersion: null, gitPath: "/usr/bin/git" });
    vi.mocked(api.openGitDownload).mockResolvedValue(null);
    renderStart();
    await userEvent.click(await screen.findByRole("button", { name: "내려받기" }));
    expect(api.openGitDownload).toHaveBeenCalled();
  });

  it.each(["ok", "distroManaged"] as const)("%s이면 안내가 없다", async (gitSecurity) => {
    vi.mocked(api.recentProjects).mockResolvedValue([]);
    vi.mocked(api.appStatus).mockResolvedValue({ gitVersion: "2.34.1", gitSecurity, gitPackageVersion: null, gitPath: "/usr/bin/git" });
    renderStart();
    await screen.findByText("폴더 열기", { exact: false });
    expect(screen.queryByRole("button", { name: "내려받기" })).toBeNull();
  });
});
