import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: { executeAction: vi.fn() },
}));

import { AppProviders, createQueryClient } from "../../App";
import { ApiError, api } from "../../lib/api";
import { CommitBox } from "./CommitBox";

function renderBox(includedCount: number) {
  vi.mocked(api.executeAction).mockResolvedValue({
    kind: "committed",
    shortOid: "abc1234",
    summary: "로그인 화면",
  });
  render(
    <AppProviders client={createQueryClient()}>
      <CommitBox folder="/repo" includedCount={includedCount} />
    </AppProviders>,
  );
  return {
    input: screen.getByRole("textbox", { name: "무엇을 바꿨나요?" }),
    button: screen.getByRole("button", { name: "버전 저장" }),
  };
}

afterEach(() => vi.clearAllMocks());

describe("CommitBox", () => {
  it("넣은 파일이 없으면 꺼지고 이유를 둘째 줄로", () => {
    const { button } = renderBox(0);
    expect(button).toBeDisabled();
    expect(screen.getByText("이번 버전에 넣을 파일을 골라 주세요.")).toBeInTheDocument();
  });

  it("메시지가 비면 꺼지고, 쓰면 켜진다", async () => {
    const user = userEvent.setup();
    const { input, button } = renderBox(2);
    expect(button).toBeDisabled();
    expect(screen.getByText("무엇을 바꿨는지 적어 주세요.")).toBeInTheDocument();
    await user.type(input, "   ");
    expect(button).toBeDisabled();
    await user.type(input, "로그인 화면");
    expect(button).toBeEnabled();
  });

  it("누르면 저장하고 입력을 비운 뒤 짧은 해시를 알려준다", async () => {
    const user = userEvent.setup();
    const { input, button } = renderBox(1);
    await user.type(input, "로그인 화면");
    await user.click(button);
    expect(api.executeAction).toHaveBeenCalledWith("/repo", {
      id: "commit",
      input: { message: "로그인 화면" },
    });
    expect(await screen.findByText("버전을 저장했어요 · abc1234")).toBeInTheDocument();
    expect(input).toHaveValue("");
  });

  it("Ctrl+Enter로 저장하고, 한글 조합 중 Enter는 무시한다", async () => {
    const user = userEvent.setup();
    const { input } = renderBox(1);
    await user.type(input, "한글");
    const settle = () => new Promise((resolve) => setTimeout(resolve, 20));
    fireEvent.keyDown(input, { key: "Enter", ctrlKey: true, isComposing: true });
    fireEvent.keyDown(input, { key: "Enter" });
    await settle();
    expect(api.executeAction).not.toHaveBeenCalled();
    fireEvent.keyDown(input, { key: "Enter", ctrlKey: true });
    await vi.waitFor(() => expect(api.executeAction).toHaveBeenCalledTimes(1));
  });

  it("이름·이메일이 없으면 안내", async () => {
    const user = userEvent.setup();
    const { input, button } = renderBox(1);
    vi.mocked(api.executeAction).mockRejectedValue(
      new ApiError({ code: "identityMissing", detail: "Please tell me who you are." }),
    );
    await user.type(input, "저장");
    await user.click(button);
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "버전에 남길 이름과 이메일이 설정되어 있지 않아요.",
    );
    expect(input).toHaveValue("저장");
  });
});
