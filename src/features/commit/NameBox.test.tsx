import { act, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: {
    executeAction: vi.fn(),
    secretCheck: vi.fn(),
    identity: vi.fn(async () => ({ name: null, email: null, complete: false, noreplyEmail: null })),
  },
}));

import { AppProviders, createQueryClient } from "../../App";
import { ApiError, api, type ChangedFile } from "../../lib/api";
import { useUndoStore } from "../undo/undoStore";
import { NameBox, composeMessage, splitMessage } from "./NameBox";

const files: ChangedFile[] = [
  { path: "src/화면.tsx", origPath: null, change: "modified", staged: false },
  { path: "src/비밀번호.ts", origPath: null, change: "untracked", staged: false },
  { path: "img/배경.png", origPath: null, change: "modified", staged: false },
];

function show(list = files, onSaved = vi.fn(), focusAfterSave?: () => void) {
  vi.mocked(api.executeAction).mockResolvedValue({ kind: "committed", shortOid: "abc1234", summary: "x" });
  vi.mocked(api.secretCheck).mockResolvedValue({ findings: [], skipped: 0 });
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

  it("이름·이메일이 없어 막히면 그 자리에서 정한다(ux.md 높음)", async () => {
    show();
    vi.mocked(api.executeAction).mockRejectedValueOnce(new ApiError({ code: "identityMissing", detail: null }));
    await userEvent.click(screen.getByLabelText("버전 이름"));
    await userEvent.keyboard("{Control>}{Enter}{/Control}");
    const form = await screen.findByRole("form", { name: "이름과 이메일" });
    expect(screen.getByRole("alert")).toHaveTextContent("이름과 이메일이 설정되어 있지 않아요");
    // 입력칸이 바로 아래 있으니 설정을 여는 공용 단추는 없다.
    expect(screen.queryByRole("button", { name: "이름과 이메일 설정" })).toBeNull();
    vi.mocked(api.executeAction).mockResolvedValueOnce({ kind: "identitySaved" });
    await userEvent.type(screen.getByLabelText("이름"), "김지원");
    await userEvent.type(screen.getByLabelText("이메일"), "ji@example.com");
    await userEvent.click(screen.getByRole("button", { name: "저장" }));
    await vi.waitFor(() =>
      expect(api.executeAction).toHaveBeenLastCalledWith("", {
        id: "identity-set",
        input: { name: "김지원", email: "ji@example.com" },
      }),
    );
    await vi.waitFor(() => expect(form).not.toBeInTheDocument());
    expect(screen.queryByRole("alert")).toBeNull();
  });

  it("비워 두고 Ctrl+Enter면 제안 이름으로, 고친 파일을 모두 넣고 저장한다", async () => {
    const onSaved = show();
    const name = screen.getByLabelText("버전 이름");
    expect(name).toHaveAttribute("placeholder", "화면.tsx 외 2개 고침");
    await userEvent.click(name);
    await userEvent.keyboard("{Control>}{Enter}{/Control}");
    expect(api.secretCheck).toHaveBeenCalledWith("/repo", true);
    await vi.waitFor(() =>
      expect(api.executeAction).toHaveBeenCalledWith("/repo", {
        id: "commit",
        input: { message: "화면.tsx 외 2개 고침", stageAll: true },
      }),
    );
    await vi.waitFor(() => expect(onSaved).toHaveBeenCalledWith("저장했어요"));
  });

  it("이름과 설명은 두 줄 메시지, 넣은 파일이 있으면 모두 넣지 않는다", async () => {
    const focus = vi.fn();
    show([{ ...files[0]!, staged: true }, files[1]!], vi.fn(), focus);
    await userEvent.type(screen.getByLabelText("버전 이름"), "로그인 입력 검사");
    await userEvent.type(screen.getByLabelText("설명 추가"), "짧은 비밀번호 막음");
    await userEvent.click(screen.getByRole("button", { name: /버전 저장/ }));
    await vi.waitFor(() =>
      expect(api.executeAction).toHaveBeenCalledWith("/repo", {
        id: "commit",
        input: { message: "로그인 입력 검사\n\n짧은 비밀번호 막음", stageAll: false },
      }),
    );
    await vi.waitFor(() => expect(focus).toHaveBeenCalled());
    expect(screen.getByLabelText("버전 이름")).toHaveValue("");
  });

  it("키로 보이는 파일이 있으면 저장하지 않고 알림 줄, 다시 저장하면 저장한다", async () => {
    show();
    vi.mocked(api.secretCheck).mockResolvedValue({
      findings: [{ path: "src/비밀번호.ts", rule: "githubToken", line: 3 }],
      skipped: 0,
    });
    await userEvent.click(screen.getByRole("button", { name: /버전 저장/ }));
    expect(await screen.findByText("키나 비밀번호로 보이는 파일 1개")).toBeInTheDocument();
    expect(screen.getByText("src/비밀번호.ts")).toBeInTheDocument();
    expect(screen.getByText(/GitHub 토큰/)).toBeInTheDocument();
    expect(api.executeAction).not.toHaveBeenCalled();
    // 새 파일만 걸렸으니 제외할 파일에 추가가 있고, 넣은 파일이 없으니 빼기는 없다.
    expect(screen.getByRole("button", { name: "제외할 파일에 추가" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /이번 버전에서 빼기/ })).not.toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "그래도 저장" }));
    await vi.waitFor(() =>
      expect(api.executeAction).toHaveBeenCalledWith("/repo", expect.objectContaining({ id: "commit" })),
    );
    expect(api.secretCheck).toHaveBeenCalledTimes(1);
  });

  it("제외할 파일에 추가는 ignore를 부르고 되돌리기 토스트를 띄운다", async () => {
    const onSaved = show();
    vi.mocked(api.secretCheck).mockResolvedValue({
      findings: [{ path: "src/비밀번호.ts", rule: "envFile", line: null }],
      skipped: 0,
    });
    vi.mocked(api.executeAction).mockResolvedValue({ kind: "ignored", count: 1 });
    await userEvent.click(screen.getByRole("button", { name: /버전 저장/ }));
    await userEvent.click(await screen.findByRole("button", { name: "제외할 파일에 추가" }));
    expect(api.executeAction).toHaveBeenCalledWith("/repo", { id: "ignore", input: { paths: ["src/비밀번호.ts"] } });
    await vi.waitFor(() => expect(onSaved).toHaveBeenCalledWith("제외할 파일에 넣었어요 · 1개"));
    expect(screen.queryByText("키나 비밀번호로 보이는 파일 1개")).not.toBeInTheDocument();
  });

  it("넣은 파일이 걸리면 이번 버전에서 빼기를 보인다", async () => {
    show([{ ...files[0]!, staged: true }]);
    vi.mocked(api.secretCheck).mockResolvedValue({
      findings: [{ path: "src/화면.tsx", rule: "apiKey", line: 1 }],
      skipped: 0,
    });
    await userEvent.click(screen.getByRole("button", { name: /버전 저장/ }));
    await userEvent.click(await screen.findByRole("button", { name: /이번 버전에서 빼기/ }));
    expect(api.executeAction).toHaveBeenCalledWith("/repo", { id: "unstage", input: { paths: ["src/화면.tsx"] } });
    expect(screen.queryByRole("button", { name: "제외할 파일에 추가" })).not.toBeInTheDocument();
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
