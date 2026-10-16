import { act, render as rtlRender, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactElement } from "react";
import { describe, expect, it, vi } from "vitest";

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: {
    branches: vi.fn(async () => ({ local: [], remote: [], defaultBranch: "main", orphanParked: [], remotes: [] })),
    githubStatus: vi.fn(async () => ({ configured: true, connected: false, login: null, remembered: false, workflow: false, email: false })),
    hostingContext: vi.fn(async () => ({
      provider: "github",
      repo: "octo/hello",
      configured: true,
      connected: false,
      branch: "main",
      head: "main",
      base: "main",
      isDefaultBranch: true,
      viaFork: false,
    })),
  },
}));

import { AppProviders, createQueryClient } from "../../App";
import { api } from "../../lib/api";
import type { StatusView } from "../../lib/api";
import { useSettingsStore } from "../../features/settings/settingsStore";
import { useRepoStore } from "../../state/repoStore";
import { TopBar as RealTopBar } from "./TopBar";

const render = (ui: ReactElement) => rtlRender(<AppProviders client={createQueryClient()}>{ui}</AppProviders>);
const TopBar = (props: Omit<Parameters<typeof RealTopBar>[0], "onDone">) => <RealTopBar {...props} onDone={vi.fn()} />;

const repo = { path: "/home/me/내 프로젝트", name: "내 프로젝트" };
const status = (branch: Partial<StatusView["branch"]>): StatusView => ({
  branch: { name: "main", hasCommits: true, upstream: null, ahead: 0, behind: 0, ...branch },
  files: [],
});

describe("TopBar", () => {
  it("프로젝트 이름과 브랜치를 보여준다", async () => {
    const user = userEvent.setup();
    render(<TopBar repo={repo} status={status({})} />);
    expect(screen.getByRole("heading", { name: "내 프로젝트" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /main/ })).toHaveAttribute("aria-keyshortcuts", "Control+B");
    // 프로젝트 이름은 프로젝트 메뉴 단추이고, 경로는 툴팁이다.
    const project = screen.getByRole("button", { name: "프로젝트 메뉴: 내 프로젝트" });
    expect(project).not.toHaveAttribute("title");
    await user.hover(project);
    expect(await screen.findByRole("tooltip")).toHaveTextContent(repo.path);
  });

  it("오른쪽 버전 기록 단추는 Ctrl+H이고 누르면 화면을 바꾼다", async () => {
    render(<TopBar repo={repo} status={status({})} />);
    const button = screen.getByRole("button", { name: "버전 기록" });
    expect(button).toHaveAttribute("aria-keyshortcuts", "Control+H");
    expect(button).toHaveAttribute("aria-pressed", "false");
    await userEvent.click(button);
    expect(useRepoStore.getState().view).toBe("history");
    act(() => useRepoStore.setState({ view: "home" }));
  });

  it("오른쪽 컨트롤은 한 덩어리로 묶여 함께 줄이 바뀐다", () => {
    render(<TopBar repo={repo} status={status({})} />);
    const right = screen.getByRole("button", { name: "설정" }).parentElement!;
    expect(right).toContainElement(screen.getByRole("button", { name: "버전 기록" }));
    expect(right).toContainElement(screen.getByRole("button", { name: "작업 내역" }));
    expect(right).not.toContainElement(screen.getByRole("button", { name: /main/ }));
  });

  it("특정 버전을 보는 중이면 브랜치 이름 대신 안내", () => {
    render(<TopBar repo={repo} status={status({ name: null })} />);
    expect(screen.getByText("특정 버전을 보는 중")).toBeInTheDocument();
  });

  it("설정 단추와, 원격이 없고 연결 안 됐으면 GitHub 연결 단추", async () => {
    render(<TopBar repo={repo} status={status({})} />);
    await userEvent.click(await screen.findByRole("button", { name: "GitHub 연결" }));
    expect(useSettingsStore.getState()).toMatchObject({ open: true, connect: true });
    act(() => useSettingsStore.getState().close());
    await userEvent.click(screen.getByRole("button", { name: "설정" }));
    expect(useSettingsStore.getState()).toMatchObject({ open: true, connect: false });
    act(() => useSettingsStore.getState().close());
  });

  it("오른쪽 컨트롤은 다섯 개를 넘지 않는다", async () => {
    const { container } = render(<TopBar repo={repo} status={status({})} />);
    await screen.findByRole("button", { name: "GitHub 연결" });
    // 오른쪽 묶음(헤더의 둘째 칸)의 컨트롤 수
    const right = container.querySelector("header")!.children[1]!;
    expect(right.children.length).toBeLessThanOrEqual(5);
  });

  it("GitHub 원격이면 검토 요청 단추, 오른쪽은 다섯 개", async () => {
    vi.mocked(api.branches).mockResolvedValue({
      local: [],
      remote: [],
      defaultBranch: "main",
      orphanParked: [],
      remotes: [{ name: "origin", url: "https://github.com/octo/hello.git" }],
    } as unknown as Awaited<ReturnType<typeof api.branches>>);
    const { container } = render(<TopBar repo={repo} status={status({ upstream: "origin/main" })} />);
    const pr = await screen.findByRole("button", { name: "검토 요청" });
    expect(screen.queryByRole("button", { name: "GitHub 연결" })).toBeNull();
    await userEvent.click(pr);
    expect(useRepoStore.getState().view).toBe("pr");
    expect(pr).toHaveAttribute("aria-pressed", "true");
    expect(container.querySelector("header")!.children[1]!.children.length).toBeLessThanOrEqual(5);
    act(() => useRepoStore.setState({ view: "home" }));
  });
});
