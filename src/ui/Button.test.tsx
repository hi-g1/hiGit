import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { Button } from "./Button";

describe("Button 꺼진 까닭(D-191)", () => {
  it("까닭이 있으면 aria-disabled로 포커스를 받고 까닭을 말하며 누름은 무시한다", async () => {
    const onClick = vi.fn();
    render(
      <Button disabledReason="고친 파일이 없어요" onClick={onClick}>
        저장
      </Button>,
    );
    const button = screen.getByRole("button", { name: "저장" });
    expect(button).toHaveAttribute("aria-disabled", "true");
    expect(button).not.toBeDisabled();
    expect(button).toHaveAccessibleDescription("고친 파일이 없어요");
    await userEvent.tab();
    expect(button).toHaveFocus();
    await userEvent.click(button);
    expect(onClick).not.toHaveBeenCalled();
  });

  it("까닭이 없으면 평소 단추", async () => {
    const onClick = vi.fn();
    render(<Button onClick={onClick}>저장</Button>);
    const button = screen.getByRole("button", { name: "저장" });
    expect(button).not.toHaveAttribute("aria-disabled");
    await userEvent.click(button);
    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it("진행 중 꺼짐은 disabled 그대로", () => {
    render(<Button disabled>저장</Button>);
    expect(screen.getByRole("button", { name: "저장" })).toBeDisabled();
  });
});
