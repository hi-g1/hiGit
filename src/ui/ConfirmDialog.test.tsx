import { act, fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { useShortcut } from "../lib/useShortcut";
import { ConfirmDialog } from "./ConfirmDialog";

function WithUndo({ onUndo, ...props }: { onUndo: () => void; onConfirm: () => void; onClose: () => void; pending?: boolean }) {
  useShortcut("undo", onUndo);
  useShortcut("sync", onUndo);
  return <ConfirmDialog open title="이 파일 고친 것 버리기" subtitle="a.txt" dangerLabel="버리기" closeLabel="닫기" {...props} />;
}

describe("확인 대화상자", () => {
  it("포커스는 닫기에, Enter는 위험 버튼을 누르지 않는다", () => {
    const onConfirm = vi.fn();
    const onClose = vi.fn();
    render(<WithUndo onUndo={vi.fn()} onConfirm={onConfirm} onClose={onClose} />);
    const close = screen.getByRole("button", { name: "닫기" });
    expect(close).toHaveFocus();
    expect(screen.getByRole("dialog")).toHaveAttribute("aria-modal", "true");
    fireEvent.click(close);
    expect(onClose).toHaveBeenCalledTimes(1);
    expect(onConfirm).not.toHaveBeenCalled();
  });

  it("열린 동안 Ctrl+Z·Ctrl+Shift+S 같은 전역 단축키가 돌지 않는다", () => {
    const onUndo = vi.fn();
    render(<WithUndo onUndo={onUndo} onConfirm={vi.fn()} onClose={vi.fn()} />);
    const close = screen.getByRole("button", { name: "닫기" });
    fireEvent.keyDown(close, { code: "KeyZ", key: "z", ctrlKey: true });
    fireEvent.keyDown(close, { code: "KeyS", key: "S", ctrlKey: true, shiftKey: true });
    // 대화상자 밖(창)에 직접 온 키도 쉰다.
    act(() => {
      window.dispatchEvent(new KeyboardEvent("keydown", { code: "KeyZ", key: "z", ctrlKey: true, cancelable: true }));
    });
    expect(onUndo).not.toHaveBeenCalled();
  });

  it("실행 중이면 위험 버튼이 꺼진다", () => {
    render(<WithUndo onUndo={vi.fn()} onConfirm={vi.fn()} onClose={vi.fn()} pending />);
    expect(screen.getByRole("button", { name: "버리기" })).toBeDisabled();
  });
});
