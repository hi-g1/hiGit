import { act, render as rtlRender, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactElement } from "react";
import { describe, expect, it, vi } from "vitest";

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: {
    branches: vi.fn(async () => ({ local: [], remote: [], defaultBranch: "main", orphanParked: [], remotes: [] })),
  },
}));

import { AppProviders, createQueryClient } from "../../App";
import type { StatusView } from "../../lib/api";
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
  it("프로젝트 이름과 브랜치를 보여준다", () => {
    render(<TopBar repo={repo} status={status({})} />);
    expect(screen.getByRole("heading", { name: "내 프로젝트" })).toBeInTheDocument();
    // 프로젝트 이름은 프로젝트 메뉴 단추이고, 경로는 툴팁이다.
    expect(screen.getByRole("button", { name: "프로젝트 메뉴: 내 프로젝트" })).toHaveAttribute("title", repo.path);
    expect(screen.getByRole("button", { name: /main/ })).toHaveAttribute("aria-keyshortcuts", "Control+B");
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

  it("특정 버전을 보는 중이면 브랜치 이름 대신 안내", () => {
    render(<TopBar repo={repo} status={status({ name: null })} />);
    expect(screen.getByText("특정 버전을 보는 중")).toBeInTheDocument();
  });
});
