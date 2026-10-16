import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: { mergeHint: vi.fn(), executeAction: vi.fn() },
}));

import { AppProviders, createQueryClient } from "../../App";
import { ApiError, api, toApiError, type MergeHintView } from "../../lib/api";
import { useToastStore } from "../../ui/toastStore";
import { MergeHint } from "./MergeHint";

const hint = (over: Partial<MergeHintView> = {}): MergeHintView => ({
  kind: "bring",
  branch: "main",
  behind: 3,
  ahead: 1,
  overlap: [],
  overlapCount: 0,
  ...over,
});

function show(h: MergeHintView, onDone = vi.fn()) {
  vi.mocked(api.mergeHint).mockResolvedValue(h);
  render(
    <AppProviders client={createQueryClient()}>
      <MergeHint repo="/repo" onDone={onDone} />
    </AppProviders>,
  );
  return onDone;
}

afterEach(() => {
  vi.clearAllMocks();
  useToastStore.setState({ toast: null });
});

describe("합치기 제안", () => {
  it("기본 브랜치에 새 버전이 있으면 개수와 가져와 합치기", async () => {
    const onDone = show(hint());
    expect(await screen.findByText("main에 새 버전 3개")).toBeInTheDocument();
    vi.mocked(api.executeAction).mockResolvedValue({ kind: "merged", branch: "main", fastForward: false });
    await userEvent.click(screen.getByRole("button", { name: "main 가져와 합치기" }));
    expect(api.executeAction).toHaveBeenCalledWith("/repo", { id: "merge", input: { branch: "main", locale: "ko" } });
    await vi.waitFor(() => expect(onDone).toHaveBeenCalledWith("합쳤어요"));
  });

  it("겹친 파일 수를 옅게 보이고, 겹치면 토스트 없이 충돌 화면으로", async () => {
    const onDone = show(hint({ overlap: ["a.txt", "b.txt"], overlapCount: 2 }));
    expect(await screen.findByText("겹친 파일 2개")).toBeInTheDocument();
    vi.mocked(api.executeAction).mockResolvedValue({ kind: "mergeConflicts", paths: ["a.txt"] });
    await userEvent.click(screen.getByRole("button", { name: "main 가져와 합치기" }));
    await vi.waitFor(() => expect(api.executeAction).toHaveBeenCalled());
    expect(onDone).not.toHaveBeenCalled();
  });

  it("실패는 짧은 문장 토스트", async () => {
    show(hint());
    vi.mocked(api.executeAction).mockRejectedValue(new ApiError({ code: "stagedChanges", detail: null }));
    await userEvent.click(await screen.findByRole("button", { name: "main 가져와 합치기" }));
    await vi.waitFor(() =>
      expect(toApiError(useToastStore.getState().toast?.error?.value).code).toBe("stagedChanges"),
    );
  });

  it("내 쪽만 앞서 있거나 제안이 없으면 노선도 옆에는 아무것도 없다", async () => {
    show(hint({ kind: "into", behind: 0 }));
    await vi.waitFor(() => expect(api.mergeHint).toHaveBeenCalled());
    expect(screen.queryByRole("button")).not.toBeInTheDocument();
  });
});
