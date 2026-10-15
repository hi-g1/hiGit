import { act, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: { conflictFile: vi.fn(), executeAction: vi.fn() },
}));

import { AppProviders, createQueryClient } from "../../App";
import { useConflictStore } from "../../features/conflict/conflictStore";
import { api, type ConflictFileView, type ConflictStateView } from "../../lib/api";
import { Conflict } from "./Conflict";

const state: ConflictStateView = {
  active: true,
  context: "merge",
  ours: { label: "로그인 기능", author: "준호", at: 1_791_000_000, summary: "비밀번호 확인 추가", shortOid: "aaa1111" },
  theirs: { label: "main", author: "민지", at: 1_791_100_000, summary: "버튼 문구 통일", shortOid: "bbb2222" },
  files: [
    { path: "src/화면.tsx", kind: "text", hunks: 2 },
    { path: "그림.png", kind: "whole", hunks: 1 },
  ],
  moreFiles: 0,
  cleanFiles: 5,
  otherOperation: false,
};

const textFile: ConflictFileView = {
  path: "src/화면.tsx",
  kind: "text",
  hunks: [
    { index: 0, line: 14, ours: ['<Form title="로그인하기" />'], base: ['<Form title="로그인" />'], theirs: ['<Form submitLabel="들어가기" />'], before: ["앞 줄"], after: ["뒤 줄"] },
    { index: 1, line: 30, ours: ["내 둘째"], base: ["원래 둘째"], theirs: ["남 둘째"], before: [], after: [] },
  ],
  oursSide: { kind: "text" },
  theirsSide: { kind: "text" },
  oursPreview: [],
  theirsPreview: [],
};

const binaryFile: ConflictFileView = {
  path: "그림.png",
  kind: "whole",
  hunks: [],
  oursSide: { kind: "binary" },
  theirsSide: { kind: "deleted" },
  oursPreview: [],
  theirsPreview: [],
};

function show(onDone = vi.fn()) {
  vi.mocked(api.conflictFile).mockImplementation(async (_repo, path) => (path === "그림.png" ? binaryFile : textFile));
  vi.mocked(api.executeAction).mockResolvedValue({ kind: "conflictFinished" });
  render(
    <AppProviders client={createQueryClient()}>
      <Conflict repo="/repo" state={state} onDone={onDone} />
    </AppProviders>,
  );
  return onDone;
}

afterEach(() => {
  vi.clearAllMocks();
  act(() => useConflictStore.getState().clear());
  window.localStorage.clear();
});

describe("충돌 해결 화면", () => {
  it("이름표·남은 곳·질문을 보여주고 표시 문자는 없다", async () => {
    show();
    expect(screen.getByText("충돌 3곳 중 3곳 남음")).toBeInTheDocument();
    expect(await screen.findByText("두 브랜치가 같은 줄을 다르게 고쳤어요. 어느 쪽을 남길까요?")).toBeInTheDocument();
    expect(await screen.findByText('<Form title="로그인하기" />')).toBeInTheDocument();
    expect(screen.getByText(/준호 · .* · 비밀번호 확인 추가/)).toBeInTheDocument();
    expect(screen.getByText("src/화면.tsx")).toBeInTheDocument();
    expect(screen.getByText("14번째 줄", { exact: false })).toBeInTheDocument();
    expect(screen.getByText("문제없이 합쳐진 5개")).toBeInTheDocument();
    expect(document.body.textContent).not.toMatch(/<<<<<<<|>>>>>>>|=======/);
    // 다 고르기 전에는 마치기 단추가 없다.
    expect(screen.queryByRole("button", { name: "합치기 마치기" })).not.toBeInTheDocument();
  });

  it("고르면 저장될 모습이 바뀌고, 1·2 키로도 고른다", async () => {
    show();
    await screen.findByText('<Form title="로그인하기" />');
    const preview = screen.getByRole("region", { name: "저장될 모습" });
    expect(within(preview).queryByText('<Form submitLabel="들어가기" />')).not.toBeInTheDocument();
    const useThis = screen.getAllByRole("button", { name: "이걸로" });
    await userEvent.click(useThis[1]!);
    expect(within(preview).getByText('<Form submitLabel="들어가기" />')).toBeInTheDocument();
    expect(screen.getByText("충돌 3곳 중 2곳 남음")).toBeInTheDocument();
    await act(async () => {
      window.dispatchEvent(new KeyboardEvent("keydown", { code: "Digit1", key: "1", bubbles: true }));
    });
    expect(within(preview).getByText('<Form title="로그인하기" />')).toBeInTheDocument();
  });

  it("모든 곳을 고르면 합치기 마치기가 나타나 conflict-finish를 부른다", async () => {
    const onDone = show();
    await screen.findByText('<Form title="로그인하기" />');
    await userEvent.click(screen.getByRole("button", { name: "둘 다 쓰기" }));
    await userEvent.click(screen.getByRole("button", { name: /다음 곳/ }));
    await screen.findByText("내 둘째");
    await userEvent.click(screen.getAllByRole("button", { name: "이걸로" })[0]!);
    await userEvent.click(screen.getByRole("button", { name: /다음 곳/ }));
    expect(await screen.findByText("이진 파일")).toBeInTheDocument();
    expect(screen.getByText("이 쪽에서는 지웠어요")).toBeInTheDocument();
    await userEvent.click(screen.getAllByRole("button", { name: "이걸로" })[1]!);
    const finish = await screen.findByRole("button", { name: "합치기 마치기" });
    await userEvent.click(finish);
    expect(api.executeAction).toHaveBeenCalledWith("/repo", {
      id: "conflict-finish",
      input: {
        locale: "ko",
        resolutions: [
          { path: "src/화면.tsx", choice: { kind: "hunks", choices: [{ kind: "both" }, { kind: "ours" }] } },
          { path: "그림.png", choice: { kind: "theirs" } },
        ],
      },
    });
    await vi.waitFor(() => expect(onDone).toHaveBeenCalledWith("합쳤어요"));
  });

  it("직접 고치기 글상자에서는 숫자 키가 글자로 들어간다", async () => {
    show();
    await screen.findByText('<Form title="로그인하기" />');
    await userEvent.click(screen.getByRole("button", { name: "직접 고치기" }));
    const box = screen.getByRole("textbox", { name: "직접 고치기" });
    await userEvent.clear(box);
    await userEvent.type(box, "새 줄 2");
    expect(box).toHaveValue("새 줄 2");
    const preview = screen.getByRole("region", { name: "저장될 모습" });
    expect(within(preview).getByText("새 줄 2")).toBeInTheDocument();
  });

  it("합치기 그만두기는 늘 보이고 conflict-abort를 부른다", async () => {
    show();
    await userEvent.click(screen.getByRole("button", { name: "합치기 그만두기" }));
    expect(api.executeAction).toHaveBeenCalledWith("/repo", { id: "conflict-abort" });
  });
});
