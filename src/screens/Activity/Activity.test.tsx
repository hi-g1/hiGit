import { act, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: { recovery: vi.fn(), listActions: vi.fn(), executeAction: vi.fn(), openLogFolder: vi.fn(), checkBranchName: vi.fn() },
}));

import { AppProviders, createQueryClient } from "../../App";
import { api, type RecoveryItem } from "../../lib/api";
import { useRepoStore } from "../../state/repoStore";
import { Activity } from "./Activity";

const at = Math.floor(Date.now() / 1000);
const items: RecoveryItem[] = [
  { id: "j:2", at, source: "journal", action: "commit", label: "main", summary: "로그인 화면", oid: "a".repeat(40), refname: null, restore: "version", undone: false, undo: "next", beforeOid: "d".repeat(40) },
  { id: "j:1", at, source: "journal", action: "commit", label: "main", summary: "첫 화면", oid: "d".repeat(40), refname: null, restore: "version", undone: false, undo: "earlier", beforeOid: null },
  { id: "r:0", at: at - 10, source: "reflog", action: null, label: "commit: 터미널", summary: "터미널 저장", oid: "b".repeat(40), refname: null, restore: "version", undone: false, undo: "none", beforeOid: null },
  { id: "b:x", at: at - 20, source: "backup", action: null, label: "discard-main", summary: "", oid: "c".repeat(40), refname: "refs/higit/backup/x/discard-main", restore: "changes", undone: false, undo: "none", beforeOid: null },
  { id: "j:0", at: at - 30, source: "journal", action: "commit", label: "main", summary: "되돌린 저장", oid: null, refname: null, restore: "none", undone: true, undo: "done", beforeOid: null },
];

function show(onDone = vi.fn()) {
  vi.mocked(api.recovery).mockResolvedValue({ items });
  vi.mocked(api.listActions).mockResolvedValue([
    { id: "commit", glossaryId: "commit", risk: "safe", undo: "restoreRef", needsInput: true },
  ]);
  render(
    <AppProviders client={createQueryClient()}>
      <Activity repo="/repo" onDone={onDone} />
    </AppProviders>,
  );
  return onDone;
}

afterEach(() => {
  vi.clearAllMocks();
  act(() => useRepoStore.setState({ view: "home" }));
});

describe("작업 내역", () => {
  it("시간순 카드와 원천별 단추, Git 원문은 접어 둔다", async () => {
    show();
    expect(await screen.findByText("되살려도 지금 버전은 남아요")).toBeInTheDocument();
    expect(await screen.findByText("로그인 화면")).toBeInTheDocument();
    expect(screen.getByText("터미널 저장")).toBeInTheDocument();
    expect(screen.getByText("commit: 터미널").closest("details")).not.toBeNull();
    expect(screen.getAllByRole("button", { name: "이 시점으로 되돌리기" })).toHaveLength(1);
    expect(screen.getAllByRole("button", { name: "이 작업 전으로 되돌리기" })).toHaveLength(1);
    expect(screen.getByRole("button", { name: "꺼내기" })).toBeInTheDocument();
  });

  it("앱 작업 줄 아래에 Ctrl+Z가 닿는지(D-204)", async () => {
    show();
    expect(await screen.findByText("Ctrl+Z로 되돌려요")).toBeInTheDocument();
    expect(screen.getByText("위 작업을 먼저 되돌려야 해요")).toBeInTheDocument();
    expect(screen.queryByText("Ctrl+Z로 되돌릴 수 없어요")).not.toBeInTheDocument();
    // 단추가 없는 앱 작업 줄도 그 버전에서 새 브랜치를 만들 수 있다.
    expect(screen.getAllByRole("button", { name: "새 브랜치로" })).toHaveLength(3);
  });

  it("되돌리기와 꺼내기는 조회가 준 값으로 액션을 부른다", async () => {
    const onDone = show();
    vi.mocked(api.executeAction).mockResolvedValue({ kind: "restored" });
    await userEvent.click(await screen.findByRole("button", { name: "이 작업 전으로 되돌리기" }));
    expect(api.executeAction).toHaveBeenCalledWith("/repo", { id: "restore", input: { oid: "d".repeat(40) } });
    await vi.waitFor(() => expect(onDone).toHaveBeenCalledWith("되돌렸어요"));
    vi.mocked(api.executeAction).mockResolvedValue({ kind: "tookOut" });
    await userEvent.click(screen.getByRole("button", { name: "꺼내기" }));
    expect(api.executeAction).toHaveBeenCalledWith("/repo", {
      id: "take-out",
      input: { source: { kind: "ref", refname: "refs/higit/backup/x/discard-main" } },
    });
  });

  it("Esc로 홈", async () => {
    act(() => useRepoStore.setState({ view: "activity" }));
    show();
    await screen.findByText("로그인 화면");
    await userEvent.keyboard("{Escape}");
    expect(useRepoStore.getState().view).toBe("home");
  });

  it("백업 이름은 용어와 브랜치 이름으로, 모르는 이름은 그대로", async () => {
    vi.mocked(api.recovery).mockResolvedValue({
      items: [
        { ...items[3]!, id: "b:1", label: "branch-delete-기능%2F로그인" },
        { ...items[3]!, id: "b:2", label: "reset", restore: "version" },
        { ...items[3]!, id: "b:3", label: "이상한-이름" },
      ],
    });
    vi.mocked(api.listActions).mockResolvedValue([]);
    render(
      <AppProviders client={createQueryClient()}>
        <Activity repo="/repo" onDone={vi.fn()} />
      </AppProviders>,
    );
    expect(await screen.findByText("· 기능/로그인", { exact: false })).toBeInTheDocument();
    expect(screen.getByText("브랜치 지우기")).toBeInTheDocument();
    expect(screen.getByText("이 버전으로 돌아가기")).toBeInTheDocument();
    expect(screen.getByText("이상한-이름")).toBeInTheDocument();
  });

  it("되살릴 수 있는 버전은 새 브랜치로 만들 수 있다(서명 안 된 줄은 없음)", async () => {
    const onDone = show();
    vi.mocked(api.checkBranchName).mockResolvedValue({ ok: true, problem: null });
    vi.mocked(api.executeAction).mockResolvedValue({ kind: "branchCreated", name: "살림" });
    const buttons = await screen.findAllByRole("button", { name: "새 브랜치로" });
    expect(buttons).toHaveLength(3);
    await userEvent.click(buttons[0]!);
    const field = await screen.findByRole("textbox", { name: "브랜치 이름" });
    await userEvent.type(field, "살림");
    await vi.waitFor(() => expect(api.checkBranchName).toHaveBeenCalledWith("/repo", "살림"));
    await userEvent.click(screen.getByRole("button", { name: "만들기" }));
    expect(api.executeAction).toHaveBeenCalledWith("/repo", {
      id: "branch-create",
      input: { name: "살림", from: "a".repeat(40) },
    });
    await vi.waitFor(() => expect(onDone).toHaveBeenCalledWith("살림 브랜치를 만들었어요"));
  });
});
