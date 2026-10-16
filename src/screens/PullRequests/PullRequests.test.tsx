import { act, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { GitHubStatusView, PrListView, PrRowView } from "../../lib/api";

const status = vi.fn<() => Promise<GitHubStatusView>>();
vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: {
    githubStatus: () => status(),
    hostingContext: vi.fn(async () => ({
      provider: "github",
      repo: "octo/hello",
      configured: true,
      connected: true,
      branch: "feature",
      head: "feature",
      base: "main",
      isDefaultBranch: false,
    })),
    prList: vi.fn(),
    prDetail: vi.fn(),
    openHostingPage: vi.fn(async () => undefined),
  },
}));

import { AppProviders, createQueryClient } from "../../App";
import { usePrDialogStore } from "../../features/pr/prDialogStore";
import { useSettingsStore } from "../../features/settings/settingsStore";
import { api } from "../../lib/api";
import { useRepoStore } from "../../state/repoStore";
import { PullRequests } from "./PullRequests";

const REPO = "/home/me/hello";
const connected: GitHubStatusView = { configured: true, connected: true, login: "octocat", remembered: true, workflow: false, email: false };

const row = (number: number, over: Partial<PrRowView> = {}): PrRowView => ({
  number,
  title: `제목 ${number}`,
  state: "open",
  draft: false,
  head: `branch-${number}`,
  headOwner: null,
  base: "main",
  author: "mona",
  updatedAt: Math.floor(Date.now() / 1000) - 3600,
  isCurrentBranch: false,
  ...over,
});
const page = (items: PrRowView[], hasMore = false): PrListView => ({ items, hasMore });

function show() {
  return render(
    <AppProviders client={createQueryClient()}>
      <PullRequests repo={REPO} />
    </AppProviders>,
  );
}

beforeEach(() => {
  status.mockResolvedValue(connected);
  vi.mocked(api.prList).mockReset();
  vi.mocked(api.prDetail).mockReset();
  vi.mocked(api.openHostingPage).mockClear();
  act(() => useRepoStore.setState({ view: "pr", selectedPr: null }));
});

afterEach(() => {
  act(() => useSettingsStore.getState().close());
});

describe("PullRequests", () => {
  it("준비 안 됨·미연결 안내", async () => {
    status.mockResolvedValue({ ...connected, configured: false, connected: false, login: null });
    const first = show();
    expect(await screen.findByText("이 앱에는 GitHub 연결이 준비되지 않았어요")).toBeInTheDocument();
    first.unmount();

    status.mockResolvedValue({ ...connected, connected: false, login: null });
    show();
    expect(await screen.findByText("GitHub에 연결하면 검토 요청을 볼 수 있어요")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "GitHub 연결" }));
    expect(useSettingsStore.getState()).toMatchObject({ open: true, connect: true });
    expect(api.prList).not.toHaveBeenCalled();
  });

  it("줄: 번호·제목·상태 글자·브랜치·지금 브랜치, 제목은 글자로만", async () => {
    vi.mocked(api.prList).mockResolvedValue(
      page([
        row(12, { title: '<img src=x onerror="alert(1)"> 로그인 🚀', isCurrentBranch: true, head: "feature" }),
        row(11, { draft: true }),
        row(10, { state: "merged" }),
        row(9, { state: "closed", headOwner: "forker" }),
      ]),
    );
    show();
    const list = await screen.findByRole("listbox");
    const rows = within(list).getAllByRole("option");
    expect(rows).toHaveLength(4);
    expect(rows[0]).toHaveTextContent('#12<img src=x onerror="alert(1)"> 로그인 🚀열림');
    expect(rows[0]).toHaveTextContent("feature → main");
    expect(rows[0]).toHaveTextContent("지금 브랜치");
    expect(list.querySelector("img")).toBeNull();
    expect(rows[1]).toHaveTextContent("초안");
    expect(rows[2]).toHaveTextContent("합쳐짐");
    expect(rows[3]).toHaveTextContent("닫힘");
    expect(rows[3]).toHaveTextContent("forker:branch-9 → main");
    expect(screen.getByText("octo/hello")).toBeInTheDocument();
  });

  it("보기 바꾸기와 더 보기가 맞는 요청을 보낸다", async () => {
    vi.mocked(api.prList).mockImplementation(async (_repo, request) =>
      request.page === 1 ? page([row(1)], true) : page([row(2)]),
    );
    show();
    await screen.findByRole("listbox");
    expect(api.prList).toHaveBeenLastCalledWith(REPO, { state: "open", branch: null, page: 1 });
    fireEvent.click(screen.getByRole("button", { name: "더 보기" }));
    await waitFor(() => expect(screen.getAllByRole("option")).toHaveLength(2));
    expect(api.prList).toHaveBeenLastCalledWith(REPO, { state: "open", branch: null, page: 2 });
    expect(screen.queryByRole("button", { name: "더 보기" })).toBeNull();

    fireEvent.click(screen.getByRole("button", { name: "닫힘" }));
    expect(screen.getByRole("button", { name: "닫힘" })).toHaveAttribute("aria-pressed", "true");
    await waitFor(() => expect(api.prList).toHaveBeenLastCalledWith(REPO, { state: "closed", branch: null, page: 1 }));
  });

  it("고르면 상세가 펼쳐지고 GitHub에서 열기는 번호만 보낸다", async () => {
    vi.mocked(api.prList).mockResolvedValue(page([row(7), row(6)]));
    vi.mocked(api.prDetail).mockImplementation(async (_repo, number) => ({
      row: row(number),
      mergeable: number === 7 ? "conflicts" : "behind",
      checks: number === 7 ? { success: 3, failed: 1, pending: 2, partial: false } : null,
      reviews: null,
      comments: 0,
    }));
    show();
    fireEvent.click((await screen.findAllByRole("option"))[0]!);
    expect(await screen.findByText(/성공 3 · 실패 1 · 진행 2/)).toBeInTheDocument();
    expect(screen.getByText("겹침 있음")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "GitHub에서 열기" }));
    expect(api.openHostingPage).toHaveBeenCalledWith(REPO, { kind: "pullRequest", number: 7 });

    // 키보드로 다음 줄: 시험 결과를 볼 수 없으면 그 줄이 없다.
    fireEvent.keyDown(screen.getByRole("listbox"), { key: "ArrowDown" });
    expect(await screen.findByText("뒤처짐")).toBeInTheDocument();
    expect(screen.queryByText(/성공 \d/)).toBeNull();
    expect(screen.getByRole("listbox")).toHaveAttribute("aria-activedescendant", "pr-6");
  });

  it("연결이 끊겼으면 다시 연결, Esc는 홈", async () => {
    vi.mocked(api.prList).mockRejectedValue({ code: "githubExpired", detail: null });
    show();
    expect(await screen.findByText("GitHub 연결이 끊겼어요. 다시 연결해 주세요.")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "다시 연결" }));
    expect(useSettingsStore.getState().connect).toBe(true);
    fireEvent.keyDown(window, { key: "Escape" });
    expect(useRepoStore.getState().view).toBe("home");
  });

  it("빈 목록", async () => {
    vi.mocked(api.prList).mockResolvedValue(page([]));
    show();
    expect(await screen.findByText("검토 요청이 없어요")).toBeInTheDocument();
  });

  it("기본 브랜치가 아니면 새 검토 요청 단추가 대화상자를 연다", async () => {
    vi.mocked(api.prList).mockResolvedValue(page([]));
    show();
    fireEvent.click(await screen.findByRole("button", { name: "새 검토 요청" }));
    expect(usePrDialogStore.getState().open).toBe(true);
    act(() => usePrDialogStore.getState().close());
  });
});
