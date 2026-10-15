import { act, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: { executeAction: vi.fn() },
}));

import { AppProviders, createQueryClient } from "../../App";
import { api, type ChangedFile } from "../../lib/api";
import { useUndoStore } from "../undo/undoStore";
import { NameBox, composeMessage, splitMessage } from "./NameBox";

const files: ChangedFile[] = [
  { path: "src/화면.tsx", origPath: null, change: "modified", staged: false },
  { path: "src/비밀번호.ts", origPath: null, change: "untracked", staged: false },
  { path: "img/배경.png", origPath: null, change: "modified", staged: false },
];

function show(list = files, onSaved = vi.fn(), focusAfterSave?: () => void) {
  vi.mocked(api.executeAction).mockResolvedValue({ kind: "committed", shortOid: "abc1234", summary: "x" });
  render(
    <AppProviders client={createQueryClient()}>
      <NameBox folder="/repo" files={list} onSaved={onSaved} focusAfterSave={focusAfterSave} />
    </AppProviders>,
  );
  return onSaved;
}

afterEach(() => {
  vi.clearAllMocks();
  act(() => useUndoStore.setState({ restoredMessage: null }));
});

describe("버전 이름 칸", () => {
  it("메시지 조립과 나누기", () => {
    expect(composeMessage("  ", "", "제안")).toBe("제안");
    expect(composeMessage("이름", " 설명 ", "제안")).toBe("이름\n\n설명");
    expect(splitMessage("이름\n\n설명\n둘째 줄")).toEqual({ name: "이름", description: "설명\n둘째 줄" });
  });

  it("비워 두고 Ctrl+Enter면 제안 이름으로, 고친 파일을 모두 넣고 저장한다", async () => {
    const onSaved = show();
    const name = screen.getByLabelText("버전 이름");
    expect(name).toHaveAttribute("placeholder", "화면.tsx 외 2개 고침");
    await userEvent.click(name);
    await userEvent.keyboard("{Control>}{Enter}{/Control}");
    expect(api.executeAction).toHaveBeenCalledWith("/repo", {
      id: "commit",
      input: { message: "화면.tsx 외 2개 고침", stageAll: true },
    });
    await vi.waitFor(() => expect(onSaved).toHaveBeenCalledWith("저장했어요"));
  });

  it("이름과 설명은 두 줄 메시지, 넣은 파일이 있으면 모두 넣지 않는다", async () => {
    const focus = vi.fn();
    show([{ ...files[0]!, staged: true }, files[1]!], vi.fn(), focus);
    await userEvent.type(screen.getByLabelText("버전 이름"), "로그인 입력 검사");
    await userEvent.type(screen.getByLabelText("설명 추가"), "짧은 비밀번호 막음");
    await userEvent.click(screen.getByRole("button", { name: /버전 저장/ }));
    expect(api.executeAction).toHaveBeenCalledWith("/repo", {
      id: "commit",
      input: { message: "로그인 입력 검사\n\n짧은 비밀번호 막음", stageAll: false },
    });
    await vi.waitFor(() => expect(focus).toHaveBeenCalled());
    expect(screen.getByLabelText("버전 이름")).toHaveValue("");
  });

  it("한글 조합 중 Ctrl+Enter는 저장하지 않는다", async () => {
    show();
    const name = screen.getByLabelText("버전 이름");
    name.dispatchEvent(new KeyboardEvent("keydown", { code: "Enter", ctrlKey: true, isComposing: true, bubbles: true }));
    expect(api.executeAction).not.toHaveBeenCalled();
  });

  it("고친 파일이 없으면 버튼이 꺼지고 이유를 둘째 줄로", () => {
    show([]);
    expect(screen.getByRole("button", { name: /버전 저장/ })).toBeDisabled();
    expect(screen.getByText("고친 파일 없음")).toBeInTheDocument();
  });

  it("되돌린 저장의 메시지가 칸에 다시 채워진다", () => {
    show();
    act(() => useUndoStore.getState().setRestoredMessage("되돌린 이름\n\n되돌린 설명"));
    expect(screen.getByLabelText("버전 이름")).toHaveValue("되돌린 이름");
    expect(screen.getByLabelText("설명 추가")).toHaveValue("되돌린 설명");
    expect(useUndoStore.getState().restoredMessage).toBeNull();
  });
});
