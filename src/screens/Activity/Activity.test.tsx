import { act, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: { recovery: vi.fn(), listActions: vi.fn(), executeAction: vi.fn(), openLogFolder: vi.fn() },
}));

import { AppProviders, createQueryClient } from "../../App";
import { api, type RecoveryItem } from "../../lib/api";
import { useRepoStore } from "../../state/repoStore";
import { Activity } from "./Activity";

const at = Math.floor(Date.now() / 1000);
const items: RecoveryItem[] = [
  { id: "j:1", at, source: "journal", action: "commit", label: "main", summary: "로그인 화면", oid: "a".repeat(40), refname: null, restore: "version", undone: false },
  { id: "r:0", at: at - 10, source: "reflog", action: null, label: "commit: 터미널", summary: "터미널 저장", oid: "b".repeat(40), refname: null, restore: "version", undone: false },
  { id: "b:x", at: at - 20, source: "backup", action: null, label: "discard-main", summary: "", oid: "c".repeat(40), refname: "refs/higit/backup/x/discard-main", restore: "changes", undone: false },
  { id: "j:0", at: at - 30, source: "journal", action: "commit", label: "main", summary: "되돌린 저장", oid: null, refname: null, restore: "none", undone: true },
];

function show(onDone = vi.fn()) {
  vi.mocked(api.recovery).mockResolvedValue({ items });
  vi.mocked(api.listActions).mockResolvedValue([
    { id: "commit", glossaryId: "commit", risk: "safe", undo: "restoreRef" },
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
    expect(screen.getAllByRole("button", { name: "이 시점으로 되돌리기" })).toHaveLength(2);
    expect(screen.getByRole("button", { name: "꺼내기" })).toBeInTheDocument();
  });

  it("되돌리기와 꺼내기는 조회가 준 값으로 액션을 부른다", async () => {
    const onDone = show();
    vi.mocked(api.executeAction).mockResolvedValue({ kind: "restored" });
    const buttons = await screen.findAllByRole("button", { name: "이 시점으로 되돌리기" });
    await userEvent.click(buttons[0]!);
    expect(api.executeAction).toHaveBeenCalledWith("/repo", { id: "restore", input: { oid: "a".repeat(40) } });
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
});
