import { act, renderHook } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: { logStats: vi.fn() },
}));

import { api } from "../../lib/api";
import { STATS_BATCH, STATS_DELAY_MS, useCommitStats } from "./stats";

const oid = (i: number) => `${i}`.padStart(40, "0");

afterEach(() => {
  vi.useRealTimers();
  vi.clearAllMocks();
});

describe("줄 수 늦게 받기", () => {
  it("보이는 해시만 150ms 뒤 100개씩, 하나씩 묻고 다시 묻지 않는다", async () => {
    vi.useFakeTimers();
    vi.mocked(api.logStats).mockImplementation(async (_repo, oids) =>
      oids.filter((o) => o !== oid(7)).map((o) => ({ oid: o, files: 1, added: 3, removed: 2 })),
    );
    const wanted = Array.from({ length: 150 }, (_, i) => oid(i));
    const { result, rerender } = renderHook(({ w }) => useCommitStats("/repo", w), { initialProps: { w: wanted } });
    expect(api.logStats).not.toHaveBeenCalled();
    await act(async () => vi.advanceTimersByTimeAsync(STATS_DELAY_MS));
    expect(api.logStats).toHaveBeenCalledTimes(1);
    expect(vi.mocked(api.logStats).mock.calls[0]![1]).toHaveLength(STATS_BATCH);
    await act(async () => vi.advanceTimersByTimeAsync(STATS_DELAY_MS));
    expect(api.logStats).toHaveBeenCalledTimes(2);
    expect(vi.mocked(api.logStats).mock.calls[1]![1]).toHaveLength(50);
    expect(result.current.get(oid(0))).toBe(5);
    // 결과에 없던 해시는 모름이고 다시 묻지 않는다.
    expect(result.current.get(oid(7))).toBeNull();
    rerender({ w: [oid(7), oid(1)] });
    await act(async () => vi.advanceTimersByTimeAsync(STATS_DELAY_MS * 3));
    expect(api.logStats).toHaveBeenCalledTimes(2);
  });

  it("실패하면 모름으로 두고 멈추지 않는다", async () => {
    vi.useFakeTimers();
    vi.mocked(api.logStats).mockRejectedValue(new Error("x"));
    const { result } = renderHook(() => useCommitStats("/repo", [oid(1)]));
    await act(async () => vi.advanceTimersByTimeAsync(STATS_DELAY_MS));
    expect(result.current.get(oid(1))).toBeNull();
  });
});
