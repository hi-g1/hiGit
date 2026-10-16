import { QueryClient } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("../../../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../../../lib/api")>()),
  api: { executeAction: vi.fn() },
}));

import { AppProviders, createQueryClient } from "../../../../App";
import { ApiError, api } from "../../../../lib/api";
import { ko } from "../../../../messages/ko";
import type { StationInfo } from "../info";
import type { MenuCtx } from "../items";
import { TagDialog } from "./TagDialog";

const oid = "a".repeat(40);
const info: StationInfo = {
  oid,
  shortOid: "aaaaaaa",
  summary: "로그인 화면",
  author: "지원",
  time: 0,
  parents: [],
  isMerge: false,
  isRoot: true,
  isHead: true,
  onHeadLine: true,
  isAncestorOfHead: true,
  branches: [],
  tags: ["v0.9"],
  unpushed: false,
};

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
      <TagDialog info={info} ctx={ctx} onClose={onClose} />
    </AppProviders>,
  );
  return { ctx, onClose, input: screen.getByRole("textbox", { name: "태그 이름" }) };
}

afterEach(() => vi.clearAllMocks());

describe("태그 대화상자", () => {
  it("입력칸에 포커스가 가고 Enter로 만든다", async () => {
    vi.mocked(api.executeAction).mockResolvedValue({ kind: "tagged", name: "v1.0" });
    const { ctx, onClose, input } = open();
    expect(input).toHaveFocus();
    expect(screen.getByText("로그인 화면")).toBeInTheDocument();
    await userEvent.type(input, "v1.0");
    fireEvent.keyDown(input, { key: "Enter" });
    await waitFor(() => expect(onClose).toHaveBeenCalled());
    expect(api.executeAction).toHaveBeenCalledWith("/r", { id: "tag", input: { oid, name: "v1.0" } });
    expect(ctx.onDone).toHaveBeenCalledWith(ko.tagged);
  });

  it("한글 조합 중 Enter는 만들지 않는다", async () => {
    const { input } = open();
    await userEvent.type(input, "출시");
    fireEvent.keyDown(input, { key: "Enter", isComposing: true });
    fireEvent.keyDown(input, { key: "Enter", keyCode: 229 });
    expect(api.executeAction).not.toHaveBeenCalled();
  });

  it("붙은 태그의 지우기는 tag-delete를 보낸다", async () => {
    vi.mocked(api.executeAction).mockResolvedValue({ kind: "tagDeleted", name: "v0.9" });
    const { ctx, onClose } = open();
    await userEvent.click(screen.getByRole("button", { name: "지우기" }));
    expect(api.executeAction).toHaveBeenCalledWith("/r", { id: "tag-delete", input: { name: "v0.9", oid } });
    await waitFor(() => expect(onClose).toHaveBeenCalled());
    expect(ctx.onDone).toHaveBeenCalledWith(ko.tagDeleted);
  });

  it("오류는 대화상자 안에 보이고 닫지 않는다", async () => {
    vi.mocked(api.executeAction).mockRejectedValue(new ApiError({ code: "tagExists", detail: null }));
    const { ctx, onClose, input } = open();
    await userEvent.type(input, "v0.9{Enter}");
    expect(await screen.findByRole("alert")).toHaveTextContent(ko.errors.tagExists);
    expect(onClose).not.toHaveBeenCalled();
    expect(ctx.toast).not.toHaveBeenCalled();
  });

  it("Esc로 닫고, 열린 동안 키가 뒤의 전역 단축키로 가지 않는다", async () => {
    const behind = vi.fn();
    window.addEventListener("keydown", behind);
    const { onClose, input } = open();
    fireEvent.keyDown(input, { key: "z", code: "KeyZ", ctrlKey: true });
    fireEvent.keyDown(input, { key: "h", code: "KeyH", ctrlKey: true });
    expect(behind).not.toHaveBeenCalled();
    fireEvent.keyDown(input, { key: "Escape" });
    await waitFor(() => expect(onClose).toHaveBeenCalled());
    window.removeEventListener("keydown", behind);
  });
});
