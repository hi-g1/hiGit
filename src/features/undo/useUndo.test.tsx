import { act, render, renderHook, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactNode } from "react";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: { undoLast: vi.fn(), redoLast: vi.fn() },
}));

import { AppProviders, createQueryClient } from "../../App";
import { ApiError, api } from "../../lib/api";
import { TOAST_DURATION_MS } from "../../ui/Toast";
import { useToastStore } from "../../ui/toastStore";
import { useUndoStore } from "./undoStore";
import { useUndo } from "./useUndo";

const wrapper = ({ children }: { children: ReactNode }) => (
  <AppProviders client={createQueryClient()}>{children}</AppProviders>
);

const ctrlZ = (shift = false) =>
  new KeyboardEvent("keydown", { code: "KeyZ", key: "z", ctrlKey: true, shiftKey: shift, cancelable: true, bubbles: true });

afterEach(() => {
  vi.clearAllMocks();
  vi.useRealTimers();
  act(() => {
    useToastStore.setState({ toast: null });
    useUndoStore.setState({ restoredMessage: null });
  });
});

describe("되돌리기", () => {
  it("Ctrl+Z는 되돌리고 저장 메시지를 이름 칸으로 돌려준다", async () => {
    vi.mocked(api.undoLast).mockResolvedValue({ action: "commit", kind: "restoreRef", branch: "main", message: "로그인 화면", reverted: false });
    renderHook(() => useUndo("/repo"), { wrapper });
    await act(async () => {
      window.dispatchEvent(ctrlZ());
    });
    expect(api.undoLast).toHaveBeenCalledWith("/repo");
    expect(await screen.findByText("되돌렸어요")).toBeInTheDocument();
    expect(useUndoStore.getState().restoredMessage).toBe("로그인 화면");
  });

  it("글상자에 포커스가 있으면 Ctrl+Z는 글자 되돌리기에 맡긴다", async () => {
    renderHook(() => useUndo("/repo"), { wrapper });
    const box = document.createElement("textarea");
    document.body.appendChild(box);
    await act(async () => {
      box.dispatchEvent(ctrlZ());
    });
    expect(api.undoLast).not.toHaveBeenCalled();
    box.remove();
  });

  it("다시 하기는 Ctrl+Shift+Z", async () => {
    vi.mocked(api.redoLast).mockResolvedValue({ action: "commit", kind: "restoreRef", branch: "main", message: null, reverted: false });
    renderHook(() => useUndo("/repo"), { wrapper });
    await act(async () => {
      window.dispatchEvent(ctrlZ(true));
    });
    expect(api.redoLast).toHaveBeenCalled();
    expect(await screen.findByText("다시 했어요")).toBeInTheDocument();
  });

  it("올린 버전은 되돌리는 새 버전을 알리고 다시 하기 단추가 없다", async () => {
    vi.mocked(api.undoLast).mockResolvedValue({
      action: "commit",
      kind: "restoreRef",
      branch: "main",
      message: "로그인 화면",
      reverted: true,
    });
    renderHook(() => useUndo("/repo"), { wrapper });
    await act(async () => {
      window.dispatchEvent(ctrlZ());
    });
    expect(await screen.findByText("되돌리는 새 버전을 만들었어요")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /다시 하기/ })).not.toBeInTheDocument();
    expect(useUndoStore.getState().restoredMessage).toBeNull();
  });

  it.each([
    ["nothingToUndo", "되돌릴 작업이 없어요."],
    ["undoStale", "앱 밖에서 바뀌어서 되돌리지 않았어요."],
  ] as const)("%s는 짧은 문장으로 알린다", async (code, text) => {
    vi.mocked(api.undoLast).mockRejectedValue(new ApiError({ code, detail: null }));
    renderHook(() => useUndo("/repo"), { wrapper });
    await act(async () => {
      window.dispatchEvent(ctrlZ());
    });
    expect(await screen.findByText(text)).toBeInTheDocument();
  });

  it("결과 토스트의 되돌리기 버튼은 Ctrl+Z와 같은 일을 한다", async () => {
    vi.mocked(api.undoLast).mockResolvedValue({ action: "commit", kind: "restoreRef", branch: "main", message: "x", reverted: false });
    const { result } = renderHook(() => useUndo("/repo"), { wrapper });
    act(() => result.current.notifyDone("저장했어요"));
    await userEvent.click(await screen.findByRole("button", { name: /되돌리기/ }));
    expect(api.undoLast).toHaveBeenCalledTimes(1);
  });

  it("토스트는 정해진 시간 뒤 사라진다", async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    render(<div />, { wrapper });
    act(() => useToastStore.getState().show("저장했어요"));
    expect(screen.getByText("저장했어요")).toBeInTheDocument();
    await act(async () => {
      vi.advanceTimersByTime(TOAST_DURATION_MS + 500);
    });
    expect(screen.queryByText("저장했어요")).toBeNull();
  });
});
