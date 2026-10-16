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
    useToastStore.setState({ toasts: [], toast: null });
    useRepoStore.setState({ view: "home" });
  });
});

describe("비교 칸 아래 줄", () => {
  it("버리기는 확인 창 없이 하고 결과를 되돌리기 토스트로 넘긴다(D-196)", async () => {
    const onDone = show();
    vi.mocked(api.executeAction).mockResolvedValue({ kind: "discarded", count: 1 });
    await userEvent.click(screen.getByRole("button", { name: "고친 것 버리기" }));
    expect(api.previewAction).not.toHaveBeenCalled();
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    expect(api.executeAction).toHaveBeenCalledWith("/repo", { id: "discard", input: { paths: ["src/화면.tsx"] } });
    await vi.waitFor(() => expect(onDone).toHaveBeenCalledWith("버렸어요"));
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
