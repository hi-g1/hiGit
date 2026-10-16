import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { AppProviders, createQueryClient } from "../../App";
import { ApiError } from "../../lib/api";
import { ErrorNotice } from "../../ui/ErrorNotice";
import { useSettingsStore } from "../settings/settingsStore";
import { ERROR_FIX } from "./fixes";

function show(code: keyof typeof ERROR_FIX, detail: string | null = null) {
  return render(
    <AppProviders client={createQueryClient()}>
      <ErrorNotice error={new ApiError({ code, detail })} />
    </AppProviders>,
  );
}

describe("오류 안내 틀(D-194)", () => {
  it("해결 버튼이 있는 코드는 그 버튼 하나를 보인다", async () => {
    show("githubExpired");
    await userEvent.click(screen.getByRole("button", { name: "다시 연결" }));
    expect(useSettingsStore.getState()).toMatchObject({ open: true, connect: true });
    useSettingsStore.getState().close();
  });

  it("해결 버튼이 없는 코드는 버튼이 없다", () => {
    expect(ERROR_FIX.gitFailed).toBe("none");
    show("gitFailed");
    expect(screen.queryByRole("button")).toBeNull();
  });

  it("자세히 보기 안에 원문 복사", async () => {
    const write = vi.fn(async () => undefined);
    Object.defineProperty(navigator, "clipboard", { value: { writeText: write }, configurable: true });
    show("gitFailed", "fatal: 원문");
    await userEvent.click(screen.getByText("자세히 보기"));
    await userEvent.click(screen.getByRole("button", { name: "복사" }));
    expect(write).toHaveBeenCalledWith("fatal: 원문");
  });
});
