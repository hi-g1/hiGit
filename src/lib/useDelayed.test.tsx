import { act, renderHook } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { useDelayedFlag } from "./useDelayed";

afterEach(() => vi.useRealTimers());

describe("불러오는 표시 지연(D-159)", () => {
  it("300ms가 넘어야 참", () => {
    vi.useFakeTimers();
    const { result } = renderHook(() => useDelayedFlag(true));
    act(() => vi.advanceTimersByTime(299));
    expect(result.current).toBe(false);
    act(() => vi.advanceTimersByTime(2));
    expect(result.current).toBe(true);
  });

  it("그 전에 끝나면 켜지지 않고, 다시 기다리면 처음부터 센다", () => {
    vi.useFakeTimers();
    const { result, rerender } = renderHook(({ pending }) => useDelayedFlag(pending), { initialProps: { pending: true } });
    act(() => vi.advanceTimersByTime(200));
    rerender({ pending: false });
    act(() => vi.advanceTimersByTime(500));
    expect(result.current).toBe(false);
    rerender({ pending: true });
    expect(result.current).toBe(false);
    act(() => vi.advanceTimersByTime(301));
    expect(result.current).toBe(true);
    rerender({ pending: false });
    expect(result.current).toBe(false);
  });
});
