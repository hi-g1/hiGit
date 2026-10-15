import { act, renderHook } from "@testing-library/react";
import type { ReactNode } from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const handlers: { changed?: (e: unknown) => void; status?: (e: unknown) => void } = {};
vi.mock("./events", () => ({
  onRepoChanged: vi.fn(async (h: (e: unknown) => void) => {
    handlers.changed = h;
    return () => {};
  }),
  onWatchStatus: vi.fn(async (h: (e: unknown) => void) => {
    handlers.status = h;
    return () => {};
  }),
}));
vi.mock("./api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("./api")>()),
  api: { watch: vi.fn(), unwatch: vi.fn() },
}));

import { AppProviders, createQueryClient } from "../App";
import { api } from "./api";
import { useRepoWatch, WATCH_THROTTLE_MS } from "./useRepoWatch";

function setup() {
  const client = createQueryClient();
  const invalidate = vi.spyOn(client, "invalidateQueries").mockResolvedValue();
  const wrapper = ({ children }: { children: ReactNode }) => (
    <AppProviders client={client}>{children}</AppProviders>
  );
  const hook = renderHook(() => useRepoWatch("/repo"), { wrapper });
  const keys = () => invalidate.mock.calls.map((c) => (c[0] as { queryKey: unknown[] }).queryKey[0]);
  return { client, invalidate, hook, keys };
}

beforeEach(() => {
  vi.useFakeTimers();
  vi.mocked(api.watch).mockResolvedValue(null);
  vi.mocked(api.unwatch).mockResolvedValue(null);
});
afterEach(() => {
  vi.useRealTimers();
  vi.clearAllMocks();
});

describe("useRepoWatch", () => {
  it("작업 폴더 변경은 변경 사항·비교, .git 변경은 변경 사항·버전 기록을 다시 읽는다", async () => {
    const { keys, invalidate } = setup();
    await act(async () => {});
    expect(api.watch).toHaveBeenCalledWith("/repo");

    act(() => handlers.changed?.({ repo: "/repo", worktree: true, git: false }));
    expect(keys().sort()).toEqual(["conflictFile", "diff", "homeMap", "mergeState", "status"]);

    invalidate.mockClear();
    await act(async () => vi.advanceTimersByTime(WATCH_THROTTLE_MS));
    act(() => handlers.changed?.({ repo: "/repo", worktree: false, git: true }));
    expect(keys().sort()).toEqual([
      "branches",
      "conflictFile",
      "diff",
      "homeMap",
      "log",
      "mergeHint",
      "mergeState",
      "pendingWork",
      "recovery",
      "stashes",
      "status",
    ]);
  });

  it("1초 안의 여러 알림은 한 번 더로 묶는다", async () => {
    const { keys, invalidate } = setup();
    await act(async () => {});
    act(() => handlers.changed?.({ repo: "/repo", worktree: true, git: false }));
    invalidate.mockClear();
    act(() => {
      for (let i = 0; i < 10; i++) handlers.changed?.({ repo: "/repo", worktree: true, git: false });
    });
    expect(invalidate).not.toHaveBeenCalled();
    await act(async () => vi.advanceTimersByTime(WATCH_THROTTLE_MS));
    expect(keys().sort()).toEqual(["conflictFile", "diff", "homeMap", "mergeState", "status"]);
  });

  it("감시가 꺼지면 active=false, 창 포커스 새로 고침을 켠다", async () => {
    const { hook, client } = setup();
    await act(async () => {});
    act(() => handlers.status?.({ repo: "/repo", active: true, reason: null }));
    expect(hook.result.current.active).toBe(true);
    expect(client.getDefaultOptions().queries?.refetchOnWindowFocus).toBe(false);

    act(() => handlers.status?.({ repo: "/repo", active: false, reason: "한도" }));
    expect(hook.result.current.active).toBe(false);
    expect(client.getDefaultOptions().queries?.refetchOnWindowFocus).toBe(true);
  });

  it("감시를 시작하지 못하면 active=false", async () => {
    vi.mocked(api.watch).mockRejectedValue({ code: "watchUnavailable", detail: "inotify" });
    const { hook } = setup();
    await act(async () => {});
    expect(hook.result.current.active).toBe(false);
  });

  it("닫히면 감시를 끄고 창 포커스 새로 고침을 되돌린다", async () => {
    const { hook, client } = setup();
    await act(async () => {});
    act(() => handlers.status?.({ repo: "/repo", active: true, reason: null }));
    expect(client.getDefaultOptions().queries?.refetchOnWindowFocus).toBe(false);
    hook.unmount();
    await act(async () => {});
    expect(api.unwatch).toHaveBeenCalled();
    expect(client.getDefaultOptions().queries?.refetchOnWindowFocus).toBe(true);
  });

  it("watch와 unwatch는 부른 순서대로 실행된다", async () => {
    const order: string[] = [];
    let release: () => void = () => {};
    vi.mocked(api.watch).mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          release = () => {
            order.push("watch");
            resolve(null);
          };
        }),
    );
    vi.mocked(api.unwatch).mockImplementation(async () => {
      order.push("unwatch");
      return null;
    });
    const { hook } = setup();
    hook.unmount();
    await act(async () => {});
    expect(order).toEqual([]);
    release();
    await act(async () => {});
    expect(order).toEqual(["watch", "unwatch"]);
  });
});
