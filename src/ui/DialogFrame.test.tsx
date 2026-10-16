import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { composing, DialogFrame } from "./DialogFrame";

describe("DialogFrame", () => {
  it("제목·부제를 보이고 Esc로 닫는다", () => {
    const onClose = vi.fn();
    render(
      <DialogFrame title="설정" subtitle="feature → main" onClose={onClose}>
        <input aria-label="이름" />
      </DialogFrame>,
    );
    expect(screen.getByRole("dialog", { name: "설정" })).toBeInTheDocument();
    expect(screen.getByText("feature → main")).toBeInTheDocument();
    fireEvent.keyDown(screen.getByLabelText("이름"), { key: "Escape" });
    expect(onClose).toHaveBeenCalled();
  });

  it("열린 동안 키가 밖으로 나가지 않는다", () => {
    const outside = vi.fn();
    window.addEventListener("keydown", outside);
    render(
      <DialogFrame title="복제" onClose={() => {}} size="wide">
        <input aria-label="주소" />
      </DialogFrame>,
    );
    fireEvent.keyDown(screen.getByLabelText("주소"), { key: "z", ctrlKey: true });
    expect(outside).not.toHaveBeenCalled();
    window.removeEventListener("keydown", outside);
  });

  it("한글 조합 중인지 안다", () => {
    expect(composing({ nativeEvent: { isComposing: true, keyCode: 0 } as KeyboardEvent })).toBe(
      true,
    );
    expect(composing({ nativeEvent: { isComposing: false, keyCode: 229 } as KeyboardEvent })).toBe(
      true,
    );
    expect(composing({ nativeEvent: { isComposing: false, keyCode: 13 } as KeyboardEvent })).toBe(
      false,
    );
  });
});
