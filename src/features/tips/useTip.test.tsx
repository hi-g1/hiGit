import { renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: { settings: vi.fn(), setSettings: vi.fn() },
}));

import { AppProviders, createQueryClient } from "../../App";
import { api } from "../../lib/api";
import { DEFAULT_SETTINGS } from "../../lib/display";
import { useTip } from "./useTip";

const wrapper = ({ children }: { children: ReactNode }) => (
  <AppProviders client={createQueryClient()}>{children}</AppProviders>
);

afterEach(() => vi.clearAllMocks());

describe("useTip(D-205)", () => {
  it("켜져 있고 아직 닫지 않았으면 보이고, 닫으면 바로 저장한다", async () => {
    vi.mocked(api.settings).mockResolvedValue(DEFAULT_SETTINGS);
    vi.mocked(api.setSettings).mockResolvedValue({ ...DEFAULT_SETTINGS });
    const { result } = renderHook(() => useTip("firstBranch"), { wrapper });
    expect(result.current.visible).toBe(false);
    await waitFor(() => expect(result.current.visible).toBe(true));
    result.current.dismiss();
    expect(api.setSettings).toHaveBeenCalledWith({ tipSeen: "firstBranch" });
    await waitFor(() => expect(result.current.visible).toBe(false));
  });

  it("안내를 껐거나 이미 닫았으면 보이지 않는다", async () => {
    vi.mocked(api.settings).mockResolvedValue({
      ...DEFAULT_SETTINGS,
      tipsSeen: { ...DEFAULT_SETTINGS.tipsSeen, firstSync: true },
    });
    const seen = renderHook(() => useTip("firstSync"), { wrapper });
    const other = renderHook(() => useTip("firstUndo"), { wrapper });
    await waitFor(() => expect(other.result.current.visible).toBe(true));
    expect(seen.result.current.visible).toBe(false);
    vi.mocked(api.settings).mockResolvedValue({ ...DEFAULT_SETTINGS, tipsOn: false });
    const off = renderHook(() => useTip("firstUndo"), { wrapper });
    await waitFor(() => expect(api.settings).toHaveBeenCalledTimes(3));
    expect(off.result.current.visible).toBe(false);
  });
});
