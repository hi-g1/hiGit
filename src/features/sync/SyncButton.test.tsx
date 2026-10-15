import { act, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: { executeAction: vi.fn() },
}));

import { AppProviders, createQueryClient } from "../../App";
import { ApiError, api, type BranchView } from "../../lib/api";
import { useToastStore } from "../../ui/toastStore";
import { SyncButton } from "./SyncButton";
import { useSyncStore } from "./syncStore";

const branch = (over: Partial<BranchView> = {}): BranchView => ({
  name: "main",
  hasCommits: true,
  upstream: "origin/main",
  ahead: 2,
  behind: 1,
  ...over,
});

function show(b = branch(), changedFiles = 0) {
  render(
    <AppProviders client={createQueryClient()}>
      <SyncButton repo="/repo" branch={b} changedFiles={changedFiles} />
    </AppProviders>,
  );
}

afterEach(() => {
  vi.clearAllMocks();
  act(() => {
    useSyncStore.setState({ overlap: [], error: null, fetchFailedAt: null, paused: false });
    useToastStore.setState({ toast: null });
  });
});

describe("동기화 버튼", () => {
  it("보낼·받을 수를 붙이고, 고친 파일이 없으면 주 버튼", () => {
    show();
    const button = screen.getByRole("button", { name: /동기화/ });
    expect(button).toHaveTextContent("↑2");
    expect(button).toHaveTextContent("↓1");
    expect(button.className).toMatch(/primary/);
  });

  it("고친 파일이 있으면 테두리 버튼, 0은 표시하지 않는다", () => {
    show(branch({ ahead: 0, behind: 0 }), 3);
    const button = screen.getByRole("button", { name: /동기화/ });
    expect(button.className).not.toMatch(/primary/);
    expect(button).not.toHaveTextContent("↑");
  });

  it("누르거나 Ctrl+Shift+S면 동기화하고 결과를 알린다", async () => {
    vi.mocked(api.executeAction).mockResolvedValue({ kind: "synced", received: 1, sent: 2, merged: false, overlap: [] });
    show();
    await userEvent.keyboard("{Control>}{Shift>}s{/Shift}{/Control}");
    expect(api.executeAction).toHaveBeenCalledWith("/repo", { id: "sync", input: { locale: "ko" } });
    await vi.waitFor(() => expect(useToastStore.getState().toast?.text).toBe("동기화했어요 · ↑2 ↓1"));
  });

  it("겹치면 버튼 옆에 겹친 파일 수를 남긴다", async () => {
    vi.mocked(api.executeAction).mockResolvedValue({
      kind: "synced",
      received: 0,
      sent: 0,
      merged: false,
      overlap: ["a.txt", "b.txt"],
    });
    show();
    await userEvent.click(screen.getByRole("button", { name: /동기화/ }));
    expect(await screen.findByText("겹친 파일 2개")).toHaveAttribute("title", "a.txt\nb.txt");
  });

  it("오류는 안내 줄 상태로 남긴다", async () => {
    vi.mocked(api.executeAction).mockRejectedValue(new ApiError({ code: "pushRejected", detail: "x" }));
    show();
    await userEvent.click(screen.getByRole("button", { name: /동기화/ }));
    await vi.waitFor(() => expect(useSyncStore.getState().error).toBeInstanceOf(ApiError));
  });
});
