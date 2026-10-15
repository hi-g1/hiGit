import { act, render, renderHook, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

const listeners = new Map<string, (e: { payload: unknown }) => void>();
const unlisten = vi.fn();
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async (name: string, cb: (e: { payload: unknown }) => void) => {
    listeners.set(name, cb);
    return unlisten;
  }),
}));
vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: { cancelAction: vi.fn() },
}));

import { AppProviders, createQueryClient } from "../../App";
import { api } from "../../lib/api";
import { PROGRESS_GONE_MS, useProgressStore } from "./progressStore";
import { StuckNotice } from "./StuckNotice";
import { useProgress, useProgressEvents } from "./useProgress";

const tick = (action: string, elapsedSecs: number) =>
  act(() => useProgressStore.getState().receive({ repo: "/repo", action, elapsedSecs, idleSecs: elapsedSecs }));

afterEach(() => {
  vi.useRealTimers();
  vi.clearAllMocks();
  act(() => useProgressStore.getState().clear("/repo"));
});

describe("진행", () => {
  it("이벤트를 받아 두고, 구독은 화면이 사라지면 끝낸다", async () => {
    const events = renderHook(() => useProgressEvents());
    await vi.waitFor(() => expect(listeners.has("op-progress")).toBe(true));
    const progress = renderHook(() => useProgress("/repo"));
    act(() => listeners.get("op-progress")?.({ payload: { repo: "/repo", action: "push", elapsedSecs: 4, idleSecs: 1 } }));
    expect(progress.result.current).toEqual({ repo: "/repo", action: "push", elapsedSecs: 4, idleSecs: 1 });
    events.unmount();
    expect(unlisten).toHaveBeenCalled();
  });

  it("알림이 끊기면 끝난 것으로 본다", () => {
    vi.useFakeTimers();
    const progress = renderHook(() => useProgress("/repo"));
    tick("commit", 1);
    act(() => vi.advanceTimersByTime(PROGRESS_GONE_MS - 100));
    tick("commit", 2);
    act(() => vi.advanceTimersByTime(PROGRESS_GONE_MS - 100));
    expect(progress.result.current?.elapsedSecs).toBe(2);
    act(() => vi.advanceTimersByTime(200));
    expect(progress.result.current).toBeNull();
  });

  it("로컬 쓰기가 60초를 넘으면 확인 뒤 강제로 멈춘다", async () => {
    vi.mocked(api.cancelAction).mockResolvedValue(null);
    render(
      <AppProviders client={createQueryClient()}>
        <StuckNotice repo="/repo" />
      </AppProviders>,
    );
    tick("commit", 59);
    expect(screen.queryByText(/응답 없음/)).not.toBeInTheDocument();
    // 원격 작업은 동기화 버튼이 취소한다.
    tick("sync", 90);
    expect(screen.queryByText(/응답 없음/)).not.toBeInTheDocument();
    tick("commit", 61);
    expect(screen.getByText("응답 없음 · 61초")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "강제로 멈추기" }));
    expect(api.cancelAction).not.toHaveBeenCalled();
    const dialog = await screen.findByRole("dialog", { name: "강제로 멈출까요?" });
    await userEvent.click(within(dialog).getByRole("button", { name: "강제로 멈추기" }));
    expect(api.cancelAction).toHaveBeenCalledWith("/repo");
  });
});
