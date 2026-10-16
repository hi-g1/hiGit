import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { TipLine } from "./TipLine";

describe("TipLine(D-205)", () => {
  it("안내 줄이고 알겠어요가 닫는다", async () => {
    const onDismiss = vi.fn();
    render(
      <TipLine onDismiss={onDismiss} action={<button type="button">작업 내역</button>}>
        안내 문장
      </TipLine>,
    );
    const note = screen.getByRole("note");
    expect(note).toHaveTextContent("안내 문장");
    expect(screen.getByRole("button", { name: "작업 내역" })).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "알겠어요" }));
    expect(onDismiss).toHaveBeenCalledTimes(1);
  });
});
