import { fireEvent, render, screen } from "@testing-library/react";
import { useState } from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { Splitter } from "./Splitter";

// 칸들 전체 폭 960px, 1rem = 16px → 최대 30rem.
beforeEach(() => {
  vi.spyOn(HTMLElement.prototype, "getBoundingClientRect").mockReturnValue({
    width: 960,
    height: 600,
    top: 0,
    left: 0,
    right: 960,
    bottom: 600,
    x: 0,
    y: 0,
    toJSON: () => ({}),
  });
});
afterEach(() => {
  vi.restoreAllMocks();
});

function Harness({ start = 20, onCommit = () => {} }: { start?: number; onCommit?: (rem: number) => void }) {
  const [rem, setRem] = useState(start);
  return (
    <div>
      <div />
      <Splitter label="칸 폭" valueRem={rem} defaultRem={21.25} onChange={setRem} onCommit={onCommit} />
      <div />
    </div>
  );
}

const separator = () => screen.getByRole("separator", { name: "칸 폭" });

describe("칸 나눔선(D-164)", () => {
  it("separator와 범위 값", () => {
    render(<Harness />);
    const s = separator();
    expect(s).toHaveAttribute("aria-orientation", "vertical");
    expect(s).toHaveAttribute("aria-valuemin", "15");
    expect(s).toHaveAttribute("aria-valuemax", "30");
    expect(s).toHaveAttribute("aria-valuenow", "20");
    expect(s).toHaveAttribute("tabindex", "0");
  });

  it("끌면 폭이 바뀌고 놓을 때 한 번 저장, 끄는 동안 글자 선택을 막는다", () => {
    const onCommit = vi.fn();
    render(<Harness onCommit={onCommit} />);
    const s = separator();
    fireEvent.pointerDown(s, { button: 0, clientX: 100, pointerId: 1 });
    expect(document.documentElement.dataset.resizing).toBe("");
    fireEvent.pointerMove(s, { clientX: 140, pointerId: 1 });
    fireEvent.pointerMove(s, { clientX: 180, pointerId: 1 });
    expect(s).toHaveAttribute("aria-valuenow", "25");
    expect(onCommit).not.toHaveBeenCalled();
    fireEvent.pointerUp(s, { clientX: 180, pointerId: 1 });
    expect(onCommit).toHaveBeenCalledTimes(1);
    expect(onCommit).toHaveBeenCalledWith(25);
    expect(document.documentElement.dataset.resizing).toBeUndefined();
  });

  it("범위를 넘겨 끌면 15rem·50%에서 멈춘다", () => {
    const onCommit = vi.fn();
    render(<Harness onCommit={onCommit} />);
    const s = separator();
    fireEvent.pointerDown(s, { button: 0, clientX: 500, pointerId: 1 });
    fireEvent.pointerMove(s, { clientX: 0, pointerId: 1 });
    expect(s).toHaveAttribute("aria-valuenow", "15");
    fireEvent.pointerMove(s, { clientX: 2000, pointerId: 1 });
    expect(s).toHaveAttribute("aria-valuenow", "30");
    fireEvent.pointerUp(s, { pointerId: 1 });
    expect(onCommit).toHaveBeenCalledWith(30);
  });

  it("저장된 폭이 범위 밖이면(창이 줄었을 때) 범위 안으로 보인다", () => {
    render(<Harness start={50} />);
    expect(separator()).toHaveAttribute("aria-valuenow", "30");
  });

  it("더블클릭하면 기본 폭", () => {
    const onCommit = vi.fn();
    render(<Harness start={28} onCommit={onCommit} />);
    fireEvent.doubleClick(separator());
    expect(separator()).toHaveAttribute("aria-valuenow", "21.3");
    expect(onCommit).toHaveBeenCalledWith(21.25);
  });

  it("←·→가 1rem씩, 조합 중·수정키는 무시", () => {
    const onCommit = vi.fn();
    render(<Harness onCommit={onCommit} />);
    const s = separator();
    s.focus();
    fireEvent.keyDown(s, { key: "ArrowRight" });
    expect(s).toHaveAttribute("aria-valuenow", "21");
    fireEvent.keyDown(s, { key: "ArrowLeft" });
    fireEvent.keyDown(s, { key: "ArrowLeft" });
    expect(s).toHaveAttribute("aria-valuenow", "19");
    expect(onCommit).toHaveBeenLastCalledWith(19);
    fireEvent.keyDown(s, { key: "ArrowLeft", isComposing: true });
    fireEvent.keyDown(s, { key: "ArrowLeft", ctrlKey: true });
    expect(s).toHaveAttribute("aria-valuenow", "19");
  });
});
