import { act, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: { executeAction: vi.fn(), cancelAction: vi.fn() },
}));

import { AppProviders, createQueryClient } from "../../App";
import { ApiError, api, type BranchView } from "../../lib/api";
import { useToastStore } from "../../ui/toastStore";
import { useProgressStore } from "../progress/progressStore";
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
    useProgressStore.getState().clear("/repo");
  });
});

const tick = (elapsedSecs: number, idleSecs = 0, action = "sync") =>
  act(() => useProgressStore.getState().receive({ repo: "/repo", action, elapsedSecs, idleSecs }));

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
    vi.mocked(api.executeAction).mockResolvedValue({ kind: "synced", received: 1, sent: 2, merged: false, overlap: [], mergeStarted: false });
    show();
    await userEvent.keyboard("{Control>}{Shift>}s{/Shift}{/Control}");
    expect(api.executeAction).toHaveBeenCalledWith("/repo", { id: "sync", input: { locale: "ko" } });
    await vi.waitFor(() => expect(useToastStore.getState().toast?.text).toBe("동기화했어요 · ↑2 ↓1"));
  });

  it("겹쳐서 합치기를 시작하면 겹친 글자 대신 충돌 화면이 이어받는다", async () => {
    vi.mocked(api.executeAction).mockResolvedValue({
      kind: "synced",
      received: 0,
      sent: 0,
      merged: false,
      overlap: ["a.txt", "b.txt"],
      mergeStarted: true,
    });
    show();
    await userEvent.click(screen.getByRole("button", { name: /동기화/ }));
    await vi.waitFor(() => expect(api.executeAction).toHaveBeenCalled());
    expect(screen.queryByText("겹친 파일 2개")).not.toBeInTheDocument();
    expect(useSyncStore.getState().overlap).toEqual([]);
  });

  it("도는 동안 초를 보이고 3초 뒤 취소, 30초 동안 출력이 없으면 응답 없음", async () => {
    vi.mocked(api.executeAction).mockReturnValue(new Promise(() => {}));
    vi.mocked(api.cancelAction).mockResolvedValue(null);
    show();
    await userEvent.click(screen.getByRole("button", { name: /동기화/ }));
    const button = await screen.findByRole("button", { name: /동기화/ });
    await vi.waitFor(() => expect(button).toHaveTextContent("동기화중"));
    expect(button).toBeDisabled();
    expect(button).not.toHaveTextContent("↑2");

    tick(2);
    expect(button).toHaveTextContent("중 · 2초");
    expect(screen.queryByRole("button", { name: "취소" })).not.toBeInTheDocument();
    // 다른 저장소나 로컬 작업의 진행은 보이지 않는다.
    act(() => useProgressStore.getState().receive({ repo: "/other", action: "sync", elapsedSecs: 9, idleSecs: 0 }));
    expect(button).toHaveTextContent("중 · 2초");

    tick(3);
    expect(screen.queryByText(/응답 없음/)).not.toBeInTheDocument();
    tick(40, 31);
    expect(screen.getByText("응답 없음 · 31초")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "취소" }));
    expect(api.cancelAction).toHaveBeenCalledWith("/repo");
    act(() => useProgressStore.getState().clear("/other"));
  });

  it("취소하면 안내 줄 없이 토스트만", async () => {
    vi.mocked(api.executeAction).mockRejectedValue(new ApiError({ code: "cancelled", detail: null }));
    show();
    await userEvent.click(screen.getByRole("button", { name: /동기화/ }));
    await vi.waitFor(() => expect(useToastStore.getState().toast?.text).toBe("취소했어요."));
    expect(useSyncStore.getState().error).toBeNull();
  });

  it("오류는 안내 줄 상태로 남긴다", async () => {
    vi.mocked(api.executeAction).mockRejectedValue(new ApiError({ code: "pushRejected", detail: "x" }));
    show();
    await userEvent.click(screen.getByRole("button", { name: /동기화/ }));
    await vi.waitFor(() => expect(useSyncStore.getState().error).toBeInstanceOf(ApiError));
  });
});
