import { QueryClient } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("../../../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../../../lib/api")>()),
  api: { executeAction: vi.fn(), commitDetail: vi.fn() },
}));

import { AppProviders, createQueryClient } from "../../../../App";
import { ApiError, api } from "../../../../lib/api";
import { ko } from "../../../../messages/ko";
import type { StationInfo } from "../info";
import type { MenuCtx } from "../items";
import { AmendDialog } from "./AmendDialog";

const oid = "c".repeat(40);
const info: StationInfo = {
  oid,
  shortOid: "ccccccc",
  summary: "오타 있는 이름",
  author: "지원",
  time: 0,
  parents: ["d".repeat(40)],
  isMerge: false,
  isRoot: false,
  isHead: true,
  onHeadLine: true,
  isAncestorOfHead: true,
  branches: ["main"],
  tags: [],
  unpushed: true,
};

function open(stagedFiles = 0) {
  vi.mocked(api.commitDetail).mockResolvedValue({
    oid,
    shortOid: "ccccccc",
    message: "오타 있는 이름\n\n본문\n",
    author: "지원",
    time: 0,
    parents: [],
    files: [],
    moreFiles: 0,
  });
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
      <AmendDialog info={info} stagedFiles={stagedFiles} ctx={ctx} onClose={onClose} />
    </AppProviders>,
  );
  return { ctx, onClose };
}

afterEach(() => vi.clearAllMocks());

describe("마지막 버전 고치기 대화상자", () => {
  it("전체 메시지를 채우고, 바꾸면 고치기가 켜지며 expectHead와 새 메시지를 보낸다", async () => {
    vi.mocked(api.executeAction).mockResolvedValue({ kind: "amended", shortOid: "e", summary: "고친 이름" });
    const { ctx, onClose } = open();
    const box = screen.getByRole("textbox", { name: "버전 메시지" });
    await waitFor(() => expect(box).toHaveValue("오타 있는 이름\n\n본문"));
    const fix = screen.getByRole("button", { name: "고치기" });
    expect(fix).toBeDisabled();
    await userEvent.clear(box);
    await userEvent.type(box, "고친 이름");
    expect(fix).toBeEnabled();
    await userEvent.click(fix);
    await waitFor(() => expect(onClose).toHaveBeenCalled());
    expect(api.executeAction).toHaveBeenCalledWith("/r", {
      id: "amend",
      input: { expectHead: oid, message: "고친 이름" },
    });
    expect(ctx.onDone).toHaveBeenCalledWith(ko.amended);
  });

  it("넣은 파일이 있으면 메시지를 그대로 둬도 고칠 수 있고 안내가 보인다", async () => {
    vi.mocked(api.executeAction).mockResolvedValue({ kind: "amended", shortOid: "e", summary: "x" });
    open(2);
    expect(await screen.findByText(ko.amendStaged(2))).toBeInTheDocument();
    const box = screen.getByRole("textbox", { name: "버전 메시지" });
    await waitFor(() => expect(box).toHaveValue("오타 있는 이름\n\n본문"));
    // Ctrl+Enter로도 고친다(메시지는 그대로라 보내지 않음).
    fireEvent.keyDown(box, { key: "Enter", code: "Enter", ctrlKey: true });
    await waitFor(() =>
      expect(api.executeAction).toHaveBeenCalledWith("/r", { id: "amend", input: { expectHead: oid, message: null } }),
    );
  });

  it("한글 조합 중 Ctrl+Enter는 무시하고, core 오류는 대화상자 안에", async () => {
    vi.mocked(api.executeAction).mockRejectedValue(new ApiError({ code: "publishedHistory", detail: null }));
    const { onClose } = open();
    const box = screen.getByRole("textbox", { name: "버전 메시지" });
    await waitFor(() => expect(box).toHaveValue("오타 있는 이름\n\n본문"));
    await userEvent.type(box, " 더");
    fireEvent.keyDown(box, { key: "Enter", code: "Enter", ctrlKey: true, isComposing: true });
    expect(api.executeAction).not.toHaveBeenCalled();
    fireEvent.keyDown(box, { key: "Enter", code: "Enter", ctrlKey: true });
    expect(await screen.findByRole("alert")).toHaveTextContent(ko.errors.publishedHistory);
    expect(onClose).not.toHaveBeenCalled();
  });
});
