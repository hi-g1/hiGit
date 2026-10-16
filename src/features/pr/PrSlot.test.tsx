import { act, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { BranchListView, GitHubStatusView, PrListView } from "../../lib/api";

const status = vi.fn<() => Promise<GitHubStatusView>>();
vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: {
    branches: vi.fn(),
    checkBranchName: vi.fn(async () => ({ ok: true, problem: null })),
    executeAction: vi.fn(),
    mergeHint: vi.fn(async () => ({ kind: "into", branch: "main", behind: 0, ahead: 2, overlap: [], overlapCount: 0 })),
    githubStatus: () => status(),
    hostingContext: vi.fn(),
    prList: vi.fn(),
  },
}));

import { AppProviders, createQueryClient } from "../../App";
import { api } from "../../lib/api";
import { useRepoStore } from "../../state/repoStore";
import { BranchList } from "../branches/BranchList";
import { useSettingsStore } from "../settings/settingsStore";
import { usePrDialogStore } from "./prDialogStore";

const view = {
  local: [
    {
      name: "feature",
      remote: null,
      isHead: true,
      isDefault: false,
      upstream: "origin/feature",
      ahead: 0,
      behind: 0,
      gone: false,
      lastAuthor: "민지",
      lastAt: 1_700_000_000,
      parkedCount: 0,
      parkBlocked: false,
    },
  ],
  remote: [],
  defaultBranch: "main",
  orphanParked: [],
  remotes: ["origin"],
} as unknown as BranchListView;

const context = (provider: "github" | null) => ({
  provider,
  repo: provider ? "octo/hello" : null,
  configured: true,
  connected: true,
  branch: "feature",
  head: "feature",
  base: "main",
  isDefaultBranch: false,
});

const connected: GitHubStatusView = { configured: true, connected: true, login: "octocat", remembered: true, workflow: false };

async function openList() {
  render(
    <AppProviders client={createQueryClient()}>
      <BranchList repo="/repo" current="feature" onDone={vi.fn()}>
        feature
      </BranchList>
    </AppProviders>,
  );
  await userEvent.click(screen.getByRole("button", { name: /feature/ }));
}

beforeEach(() => {
  vi.mocked(api.branches).mockResolvedValue(view);
  vi.mocked(api.hostingContext).mockResolvedValue(context("github"));
  vi.mocked(api.prList).mockResolvedValue({ items: [], hasMore: false });
  status.mockResolvedValue(connected);
});

afterEach(() => {
  act(() => {
    usePrDialogStore.getState().close();
    useSettingsStore.getState().close();
    useRepoStore.setState({ view: "home", selectedPr: null });
  });
});

describe("브랜치 목록의 검토 요청", () => {
  it("GitHub 프로젝트면 검토 요청이 먼저, main에 합치기는 글자 단추", async () => {
    await openList();
    const pr = await screen.findByRole("button", { name: "검토 요청" });
    const merge = screen.getByRole("button", { name: "main에 합치기" });
    expect(pr.compareDocumentPosition(merge) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    expect(merge.className).toMatch(/text/);
    expect(api.prList).toHaveBeenCalledWith("/repo", { state: "all", branch: "feature", page: 1 });
    await userEvent.click(pr);
    expect(usePrDialogStore.getState().open).toBe(true);
  });

  it("이미 있는 요청은 번호·상태, 누르면 검토 요청 화면의 그 줄", async () => {
    const list: PrListView = {
      items: [
        {
          number: 12,
          title: "기능",
          state: "open",
          draft: false,
          head: "feature",
          headOwner: null,
          base: "main",
          author: "mona",
          updatedAt: 0,
          isCurrentBranch: true,
        },
      ],
      hasMore: false,
    };
    vi.mocked(api.prList).mockResolvedValue(list);
    await openList();
    await userEvent.click(await screen.findByRole("button", { name: "검토 요청 #12 · 열림" }));
    expect(useRepoStore.getState()).toMatchObject({ view: "pr", selectedPr: 12 });
  });

  it("연결 전이면 연결 패널을 연다", async () => {
    status.mockResolvedValue({ ...connected, connected: false, login: null });
    await openList();
    await userEvent.click(await screen.findByRole("button", { name: "검토 요청" }));
    expect(useSettingsStore.getState()).toMatchObject({ open: true, connect: true });
    expect(api.prList).not.toHaveBeenCalled();
  });

  it("GitHub가 아니면 지금처럼 main에 합치기만", async () => {
    vi.mocked(api.hostingContext).mockResolvedValue(context(null));
    await openList();
    const merge = await screen.findByRole("button", { name: "main에 합치기" });
    expect(merge.className).not.toMatch(/text/);
    const slot = merge.closest("div")!;
    expect(within(slot).queryByRole("button", { name: "검토 요청" })).toBeNull();
  });
});
