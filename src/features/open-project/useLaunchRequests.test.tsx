import { act, renderHook } from "@testing-library/react";
import type { ReactNode } from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: { openProject: vi.fn(), takeLaunchFolder: vi.fn(), recentProjects: vi.fn(), appStatus: vi.fn() },
}));
let fire: (() => void) | null = null;
vi.mock("../../lib/events", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/events")>()),
  onOpenFolder: vi.fn(async (handler: () => void) => {
    fire = handler;
    return () => {
      fire = null;
    };
  }),
}));

import { AppProviders, createQueryClient } from "../../App";
import { api } from "../../lib/api";
import { useRepoStore } from "../../state/repoStore";
import { useStartup } from "../../state/useStartup";
import { useToastStore } from "../../ui/toastStore";
import { LAUNCH_RETRY_MS, useLaunchRequests } from "./useLaunchRequests";

const wrapper = ({ children }: { children: ReactNode }) => (
  <AppProviders client={createQueryClient()}>{children}</AppProviders>
);
const opened = (path: string) => ({ kind: "opened" as const, repo: { path, name: path.split("/").pop()! } });

beforeEach(() => {
  vi.mocked(api.appStatus).mockResolvedValue({
    gitVersion: "2.45.0",
    gitSecurity: "ok",
    gitPackageVersion: null,
    gitPath: "/usr/bin/git",
    undoKey: "saved",
    undoKeyDetail: null,
  });
  vi.mocked(api.recentProjects).mockResolvedValue([{ path: "/최근", name: "최근", lastOpenedAt: 0, pinned: false, pinnedBranches: [] }]);
});

afterEach(() => {
  vi.useRealTimers();
  vi.clearAllMocks();
  document.body.replaceChildren();
  act(() => useRepoStore.setState({ repo: null, startNotice: null, pendingInit: null }));
});

describe("실행 인자로 받은 폴더(D-213)", () => {
  it("시작할 때 실행 인자가 있으면 최근 프로젝트 대신 그 폴더를 연다", async () => {
    vi.mocked(api.takeLaunchFolder).mockResolvedValue("/받은/폴더");
    vi.mocked(api.openProject).mockResolvedValue(opened("/받은/폴더"));
    const { result } = renderHook(() => useStartup(), { wrapper });
    await vi.waitFor(() => expect(result.current).toBe(true));
    expect(api.openProject).toHaveBeenCalledTimes(1);
    expect(api.openProject).toHaveBeenCalledWith("/받은/폴더", "picked");
    expect(api.recentProjects).not.toHaveBeenCalled();
    expect(useRepoStore.getState().repo?.path).toBe("/받은/폴더");
  });

  it("실행 인자의 폴더가 없으면 시작 화면에 알린다", async () => {
    vi.mocked(api.takeLaunchFolder).mockResolvedValue("/없는/폴더");
    vi.mocked(api.openProject).mockResolvedValue({ kind: "missing" });
    const { result } = renderHook(() => useStartup(), { wrapper });
    await vi.waitFor(() => expect(result.current).toBe(true));
    expect(useRepoStore.getState().startNotice).toEqual({ kind: "launchMissing", name: "" });
  });

  it("두 번째 실행이 넘긴 폴더를 열고, 작업 공간이 열려 있을 때 없는 폴더면 토스트로 알린다", async () => {
    vi.mocked(api.takeLaunchFolder).mockResolvedValue("/둘째");
    vi.mocked(api.openProject).mockResolvedValue(opened("/둘째"));
    renderHook(() => useLaunchRequests(true), { wrapper });
    await vi.waitFor(() => expect(fire).not.toBeNull());
    await act(async () => fire!());
    await vi.waitFor(() => expect(api.openProject).toHaveBeenCalledWith("/둘째", "picked"));
    await vi.waitFor(() => expect(useRepoStore.getState().repo?.path).toBe("/둘째"));

    vi.mocked(api.takeLaunchFolder).mockResolvedValue("/없음");
    vi.mocked(api.openProject).mockResolvedValue({ kind: "missing" });
    await act(async () => fire!());
    await vi.waitFor(() => expect(useToastStore.getState().toast?.text).toBe("열려던 폴더를 찾지 못했어요."));
    act(() => useToastStore.getState().clear());
    expect(useRepoStore.getState().repo?.path).toBe("/둘째");
  });

  it("확인 창이 열려 있으면 닫힌 뒤에 연다", async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    vi.mocked(api.takeLaunchFolder).mockResolvedValue("/나중");
    vi.mocked(api.openProject).mockResolvedValue(opened("/나중"));
    const dialog = document.createElement("div");
    dialog.setAttribute("role", "dialog");
    document.body.appendChild(dialog);
    renderHook(() => useLaunchRequests(true), { wrapper });
    await vi.waitFor(() => expect(fire).not.toBeNull());
    await act(async () => fire!());
    await act(async () => {
      vi.advanceTimersByTime(LAUNCH_RETRY_MS * 2);
    });
    expect(api.takeLaunchFolder).not.toHaveBeenCalled();
    dialog.remove();
    await act(async () => {
      vi.advanceTimersByTime(LAUNCH_RETRY_MS);
    });
    await vi.waitFor(() => expect(api.openProject).toHaveBeenCalledWith("/나중", "picked"));
  });
});
