import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: { settings: vi.fn(), setSettings: vi.fn() },
}));

import { AppProviders, createQueryClient } from "../../App";
import { api } from "../../lib/api";
import { DEFAULT_SETTINGS } from "../../lib/display";
import { ko } from "../../messages/ko";
import { useRepoStore } from "../../state/repoStore";
import { HomeTips } from "./HomeTips";
import { useTipStore } from "./tipStore";

function show(settings = DEFAULT_SETTINGS) {
  vi.mocked(api.settings).mockResolvedValue(settings);
  vi.mocked(api.setSettings).mockImplementation(async (p) => ({ ...settings, ...p }) as typeof settings);
  render(
    <AppProviders client={createQueryClient()}>
      <HomeTips />
    </AppProviders>,
  );
}

afterEach(() => {
  vi.clearAllMocks();
  act(() => {
    useTipStore.setState({ due: [] });
    useRepoStore.setState({ view: "home" });
  });
});

describe("처음 하는 일 안내(D-168, D-205)", () => {
  it("그 일을 한 직후에만 뜨고, 닫으면 저장되어 다시 뜨지 않는다", async () => {
    show();
    expect(screen.queryByRole("note")).not.toBeInTheDocument();
    act(() => useTipStore.getState().trigger("firstSync"));
    expect(await screen.findByRole("note")).toHaveTextContent(ko.tip.firstSync);
    await userEvent.click(screen.getByRole("button", { name: "알겠어요" }));
    expect(api.setSettings).toHaveBeenCalledWith({ tipSeen: "firstSync" });
    expect(screen.queryByRole("note")).not.toBeInTheDocument();
    // 같은 일을 또 해도 다시 뜨지 않는다.
    act(() => useTipStore.getState().trigger("firstSync"));
    expect(screen.queryByRole("note")).not.toBeInTheDocument();
  });

  it("첫 되돌리기 안내는 작업 내역 단추가 있다", async () => {
    show();
    act(() => useTipStore.getState().trigger("firstUndo"));
    await userEvent.click(await screen.findByRole("button", { name: "작업 내역" }));
    expect(useRepoStore.getState().view).toBe("activity");
  });

  it("안내를 껐으면 뜨지 않는다", async () => {
    show({ ...DEFAULT_SETTINGS, tipsOn: false });
    act(() => useTipStore.getState().trigger("firstBranch"));
    await waitFor(() => expect(api.settings).toHaveBeenCalled());
    expect(screen.queryByRole("note")).not.toBeInTheDocument();
  });
});
