import { QueryClient } from "@tanstack/react-query";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("../../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../../lib/api")>()),
  api: { executeAction: vi.fn() },
}));

import { ApiError, api } from "../../../lib/api";
import { ko } from "../../../messages/ko";
import type { MenuCtx } from "./items";
import { runAction } from "./run";

function context() {
  const queryClient = new QueryClient();
  const invalidate = vi.spyOn(queryClient, "invalidateQueries");
  const ctx: MenuCtx = {
    repo: "/r",
    toast: vi.fn(),
    queryClient,
    onDone: vi.fn(),
    ask: vi.fn(),
    openActivity: vi.fn(),
    openRewrite: vi.fn(),
    headLine: () => [],
    m: ko,
  };
  return { ctx, invalidate };
}

afterEach(() => vi.clearAllMocks());

describe("runAction", () => {
  it("성공하면 완료 문구(되돌리기는 onDone이 붙인다)와 조회 새로 고침", async () => {
    vi.mocked(api.executeAction).mockResolvedValue({ kind: "reset", dropped: 2 });
    const { ctx, invalidate } = context();
    const input = { id: "reset", input: { oid: "a".repeat(40) } } as const;
    const result = await runAction(ctx, input, { done: () => "옮겼어요" });
    expect(api.executeAction).toHaveBeenCalledWith("/r", input);
    expect(result).toEqual({ kind: "reset", dropped: 2 });
    expect(ctx.onDone).toHaveBeenCalledWith("옮겼어요");
    const keys = invalidate.mock.calls.map((c) => (c[0]?.queryKey as string[])[0]);
    expect(keys).toEqual(expect.arrayContaining(["log", "status", "branches", "recovery", "rewritePlan", "mergeState"]));
  });

  it("겹쳐 멈추면 토스트 없이 충돌 상태만 다시 읽는다", async () => {
    vi.mocked(api.executeAction).mockResolvedValue({ kind: "operationConflicts", context: "cherryPick", paths: ["a"] });
    const { ctx, invalidate } = context();
    const done = vi.fn(() => "x");
    await runAction(ctx, { id: "cherry-pick", input: { oid: "a".repeat(40) } }, { done });
    expect(done).not.toHaveBeenCalled();
    expect(ctx.onDone).not.toHaveBeenCalled();
    expect(invalidate).toHaveBeenCalledWith({ queryKey: ["mergeState", "/r"] });
  });

  it("오류 코드의 문구로 알리고, 올린 기록은 작업 내역으로 안내한다", async () => {
    const { ctx } = context();
    vi.mocked(api.executeAction).mockRejectedValue(new ApiError({ code: "branchProtected", detail: null }));
    expect(await runAction(ctx, { id: "branch-delete", input: { name: "main" } }, { done: () => null })).toBeNull();
    expect(ctx.toast).toHaveBeenCalledWith(ko.errors.branchProtected);

    vi.mocked(api.executeAction).mockRejectedValue(new ApiError({ code: "publishedHistory", detail: null }));
    await runAction(ctx, { id: "reset", input: { oid: "a".repeat(40) } }, { done: () => null });
    expect(ctx.toast).toHaveBeenLastCalledWith(ko.errors.publishedHistory, {
      label: ko.openActivity,
      onClick: ctx.openActivity,
    });
  });

  it("부른 쪽이 오류를 직접 보이면 토스트를 띄우지 않는다", async () => {
    const { ctx } = context();
    vi.mocked(api.executeAction).mockRejectedValue(new ApiError({ code: "tagExists", detail: null }));
    const onError = vi.fn();
    await runAction(ctx, { id: "tag", input: { oid: "a".repeat(40), name: "v1" } }, { done: () => null, onError });
    expect(onError).toHaveBeenCalledWith("tagExists");
    expect(ctx.toast).not.toHaveBeenCalled();
  });
});
