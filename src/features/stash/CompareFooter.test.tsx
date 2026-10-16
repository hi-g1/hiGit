import { act, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: { stashes: vi.fn(), previewAction: vi.fn(), executeAction: vi.fn() },
}));

import { AppProviders, createQueryClient } from "../../App";
import { api, type ChangedFile } from "../../lib/api";
import { useRepoStore } from "../../state/repoStore";
import { useToastStore } from "../../ui/toastStore";
import { CompareFooter } from "./CompareFooter";

const file: ChangedFile = { path: "src/화면.tsx", origPath: null, change: "modified", staged: false };

function show({ hasChanges = true, onDone = vi.fn() } = {}) {
  vi.mocked(api.stashes).mockResolvedValue({
    items: [{ oid: "a".repeat(40), message: "higit stash main", branch: "main", at: 1_791_000_000, files: 3 }],
  });
  render(
    <AppProviders client={createQueryClient()}>
      <CompareFooter repo="/repo" file={file} hasChanges={hasChanges} onDone={onDone} />
    </AppProviders>,
  );
  return onDone;
}

afterEach(() => {
  vi.clearAllMocks();
  act(() => {
    useToastStore.setState({ toast: null });
    useRepoStore.setState({ view: "home" });
  });
});

describe("비교 칸 아래 줄", () => {
  it("버리기는 파일 이름을 보여 주는 확인 없이는 실행하지 않는다", async () => {
    show();
    vi.mocked(api.previewAction).mockResolvedValue({ token: "p1" });
    await userEvent.click(screen.getByRole("button", { name: "고친 것 버리기" }));
    expect(await screen.findByText("화면.tsx의 고친 내용을 버려요.")).toBeInTheDocument();
    expect(api.executeAction).not.toHaveBeenCalled();
    // Enter가 버리기가 되지 않게 닫기에 포커스가 있다.
    expect(screen.getByRole("button", { name: "닫기" })).toHaveFocus();
    // 닫으면 실행하지 않는다.
    await userEvent.click(screen.getByRole("button", { name: "닫기" }));
    expect(api.executeAction).not.toHaveBeenCalled();
  });

  it("확인하면 토큰과 함께 버리고, 토스트의 작업 내역 단추가 그 화면을 연다", async () => {
    show();
    vi.mocked(api.previewAction).mockResolvedValue({ token: "p1" });
    vi.mocked(api.executeAction).mockResolvedValue({ kind: "discarded", count: 1 });
    await userEvent.click(screen.getByRole("button", { name: "고친 것 버리기" }));
    await userEvent.click(await screen.findByRole("button", { name: "버리기" }));
    expect(api.executeAction).toHaveBeenCalledWith("/repo", { id: "discard", input: { paths: ["src/화면.tsx"] } }, "p1");
    await vi.waitFor(() => expect(useToastStore.getState().toast?.text).toBe("버렸어요"));
    act(() => useToastStore.getState().toast?.action?.onClick());
    expect(useRepoStore.getState().view).toBe("activity");
  });

  it("임시 보관하고 목록에서 꺼낸다(고친 파일이 있으면 꺼내기는 꺼짐)", async () => {
    const onDone = show();
    vi.mocked(api.executeAction).mockResolvedValue({ kind: "stashed" });
    await userEvent.click(screen.getByRole("button", { name: "임시 보관" }));
    expect(api.executeAction).toHaveBeenCalledWith("/repo", { id: "stash" });
    await vi.waitFor(() => expect(onDone).toHaveBeenCalledWith("보관했어요"));
    await userEvent.click(await screen.findByRole("button", { name: "임시 보관 1개" }));
    // 까닭이 있는 꺼짐은 aria-disabled로 키보드에 닿고 까닭을 말한다(D-191).
    const takeOut = await screen.findByRole("button", { name: "꺼내기" });
    expect(takeOut).toHaveAttribute("aria-disabled", "true");
    // 팝오버가 열리며 포커스가 가 툴팁도 열려 있다(Radix가 aria-describedby에 툴팁 id를 덧붙인다).
    expect(takeOut.getAttribute("aria-describedby")).toBeTruthy();
    expect(screen.getAllByText("고친 파일이 없을 때 꺼낼 수 있어요").length).toBeGreaterThan(0);
  });
});
