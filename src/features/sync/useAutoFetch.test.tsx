import { act, renderHook } from "@testing-library/react";
import type { ReactNode } from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: { executeAction: vi.fn() },
}));

import { AppProviders, createQueryClient } from "../../App";
import { ApiError, api } from "../../lib/api";
import { useSyncStore } from "./syncStore";
import { FETCH_INTERVAL_MS, FOCUS_MIN_GAP_MS, useAutoFetch } from "./useAutoFetch";

const wrapper = ({ children }: { children: ReactNode }) => (
  <AppProviders client={createQueryClient()}>{children}</AppProviders>
);

beforeEach(() => vi.useFakeTimers({ shouldAdvanceTime: true }));
afterEach(() => {
  vi.useRealTimers();
  vi.clearAllMocks();
  act(() => useSyncStore.setState({ fetchFailedAt: null, fetchFailedCode: null, lastCheckedAt: null, paused: false }));
});

const fetchCalls = () =>
  vi.mocked(api.executeAction).mock.calls.filter(([, input]) => input.id === "fetch").length;

describe("저절로 하는 원격 확인", () => {
  it("열 때 한 번, 5분마다, 포커스가 돌아오면 1분이 지났을 때만", async () => {
    vi.mocked(api.executeAction).mockResolvedValue({ kind: "fetched" });
    renderHook(() => useAutoFetch("/repo", true), { wrapper });
    await vi.waitFor(() => expect(fetchCalls()).toBe(1));
    expect(api.executeAction).toHaveBeenCalledWith("/repo", { id: "fetch", input: { background: true } });
    // 성공하면 마지막 확인 시각을 남긴다(메모리만).
    await vi.waitFor(() => expect(useSyncStore.getState().lastCheckedAt).not.toBeNull());

    await act(async () => {
      window.dispatchEvent(new Event("focus"));
    });
    expect(fetchCalls()).toBe(1);
    await act(async () => {
      vi.advanceTimersByTime(FOCUS_MIN_GAP_MS + 10);
      window.dispatchEvent(new Event("focus"));
    });
    await vi.waitFor(() => expect(fetchCalls()).toBe(2));
    await act(async () => {
      vi.advanceTimersByTime(FETCH_INTERVAL_MS);
    });
    await vi.waitFor(() => expect(fetchCalls()).toBe(3));
  });

  it("원격이 없으면 하지 않는다", () => {
    renderHook(() => useAutoFetch("/repo", false), { wrapper });
    expect(fetchCalls()).toBe(0);
  });

  it("인증 실패면 실패 표시를 남기고 멈춘다", async () => {
    vi.mocked(api.executeAction).mockRejectedValue(new ApiError({ code: "httpAuthFailed", detail: null }));
    renderHook(() => useAutoFetch("/repo", true), { wrapper });
    await vi.waitFor(() => expect(useSyncStore.getState().paused).toBe(true));
    expect(useSyncStore.getState().fetchFailedAt).not.toBeNull();
    // 알림 줄이 해결 버튼을 고를 코드도 남긴다(D-197).
    expect(useSyncStore.getState().fetchFailedCode).toBe("httpAuthFailed");
    await act(async () => {
      vi.advanceTimersByTime(FETCH_INTERVAL_MS);
    });
    expect(fetchCalls()).toBe(1);
  });
});
