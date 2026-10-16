import { fireEvent, render, screen } from "@testing-library/react";
import { useState } from "react";
import { describe, expect, it, vi } from "vitest";
import { Choice } from "./Choice";

function Picker({ onChange = () => {} }: { onChange?: (v: string) => void }) {
  const [value, setValue] = useState("b");
  return (
    <Choice
      label="크기"
      value={value}
      options={[
        { value: "a", label: "작게" },
        { value: "b", label: "보통" },
        { value: "c", label: "크게" },
      ]}
      onChange={(v) => {
        setValue(v);
        onChange(v);
      }}
    />
  );
}

describe("고르는 부품", () => {
  it("radiogroup 이름과 고른 것", () => {
    render(<Picker />);
    const group = screen.getByRole("radiogroup", { name: "크기" });
    expect(group).toBeInTheDocument();
    expect(screen.getByRole("radio", { name: "보통" })).toHaveAttribute("aria-checked", "true");
    expect(screen.getByRole("radio", { name: "작게" })).toHaveAttribute("aria-checked", "false");
    // Tab은 고른 것에만 멈춘다.
    expect(screen.getByRole("radio", { name: "보통" })).toHaveAttribute("tabindex", "0");
    expect(screen.getByRole("radio", { name: "크게" })).toHaveAttribute("tabindex", "-1");
  });

  it("누르거나 방향키로 고른다", () => {
    const onChange = vi.fn();
    render(<Picker onChange={onChange} />);
    fireEvent.click(screen.getByRole("radio", { name: "크게" }));
    expect(onChange).toHaveBeenLastCalledWith("c");
    const big = screen.getByRole("radio", { name: "크게" });
    fireEvent.keyDown(big, { key: "ArrowRight" });
    expect(onChange).toHaveBeenLastCalledWith("a");
    expect(screen.getByRole("radio", { name: "작게" })).toHaveFocus();
    fireEvent.keyDown(screen.getByRole("radio", { name: "작게" }), { key: "ArrowRight", isComposing: true });
    expect(onChange).toHaveBeenCalledTimes(2);
  });
});
