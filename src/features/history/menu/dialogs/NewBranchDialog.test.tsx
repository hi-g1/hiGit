import { QueryClient } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("../../../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../../../lib/api")>()),
  api: { executeAction: vi.fn(), checkBranchName: vi.fn() },
}));

import { AppProviders, createQueryClient } from "../../../../App";
import { ApiError, api } from "../../../../lib/api";
import { ko } from "../../../../messages/ko";
import type { MenuCtx } from "../items";
import { NewBranchDialog } from "./NewBranchDialog";

const oid = "b".repeat(40);

function open() {
  const ctx: MenuCtx = {
    repo: "/r",
    toast: vi.fn(),
    queryClient: new QueryClient(),
    onDone: vi.fn(),
    ask: vi.fn(),
    openActivity: vi.fn(),
    openRewrite: vi.fn(),
    headLine: () => [],
    m: ko,
  };
  const onClose = vi.fn();
  render(
    <AppProviders client={createQueryClient()}>
      <NewBranchDialog oid={oid} shortOid="bbbbbbb" summary="옛 버전" ctx={ctx} onClose={onClose} />
    </AppProviders>,
  );
  return { ctx, onClose, input: screen.getByRole("textbox", { name: "브랜치 이름" }) };
}

afterEach(() => vi.clearAllMocks());

describe("여기서 새 브랜치 대화상자", () => {
  it("고른 버전을 from으로 보내 만든다", async () => {
    vi.mocked(api.checkBranchName).mockResolvedValue({ ok: true, problem: null });
    vi.mocked(api.executeAction).mockResolvedValue({ kind: "branchCreated", name: "다시" });
    const { ctx, onClose, input } = open();
    expect(input).toHaveFocus();
    await userEvent.type(input, "다시");
    fireEvent.keyDown(input, { key: "Enter" });
    await waitFor(() => expect(onClose).toHaveBeenCalled());
    expect(api.executeAction).toHaveBeenCalledWith("/r", { id: "branch-create", input: { name: "다시", from: oid } });
    expect(ctx.onDone).toHaveBeenCalledWith(ko.branchCreatedHere("다시"));
  });

  it("같은 이름이 있으면 안내하고 만들지 않는다", async () => {
    vi.mocked(api.checkBranchName).mockResolvedValue({ ok: false, problem: "exists" });
    const { input } = open();
    await userEvent.type(input, "main");
    expect(await screen.findByText(ko.sameBranchName)).toBeInTheDocument();
    fireEvent.keyDown(input, { key: "Enter" });
    expect(api.executeAction).not.toHaveBeenCalled();
    expect(screen.getByRole("button", { name: "만들기" })).toBeDisabled();
  });

  it("core의 오류는 대화상자 안에", async () => {
    vi.mocked(api.checkBranchName).mockResolvedValue({ ok: true, problem: null });
    vi.mocked(api.executeAction).mockRejectedValue(new ApiError({ code: "hasChanges", detail: null }));
    const { onClose, input } = open();
    await userEvent.type(input, "새것{Enter}");
    expect(await screen.findByRole("alert")).toHaveTextContent(ko.errors.hasChanges);
    expect(onClose).not.toHaveBeenCalled();
  });
});
