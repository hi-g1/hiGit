import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { describe, expect, it, vi } from "vitest";

vi.mock("./api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("./api")>()),
  api: { log: vi.fn() },
}));

import { logPage } from "../test/logPage";
import { api } from "./api";
import { invalidateLog, useLogPages } from "./queries";

describe("버전 기록 쪽", () => {
  it("다음 쪽은 첫 쪽의 끝점과 받은 수로, 무효화하면 첫 쪽만 다시", async () => {
    let calls = 0;
    vi.mocked(api.log).mockImplementation(async (_repo, q) => {
      calls += 1;
      const skip = q.skip;
      return logPage({
        commits: [{ oid: `c${skip}`.padEnd(40, "0"), shortOid: "c", parents: [], summary: "", authorName: "", authorEmail: "", authorTime: 0 }],
        hasMore: skip < 2,
        tips: q.tips ?? [`t${calls}`],
      });
    });
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    const wrapper = ({ children }: { children: ReactNode }) => <QueryClientProvider client={client}>{children}</QueryClientProvider>;
    const { result } = renderHook(() => useLogPages("/repo", "all"), { wrapper });
    await waitFor(() => expect(result.current.data?.pages).toHaveLength(1));
    for (const expected of [2, 3]) {
      await waitFor(() => expect(result.current.hasNextPage && !result.current.isFetching).toBe(true));
      await act(async () => {
        await result.current.fetchNextPage();
      });
      await waitFor(() => expect(result.current.data?.pages).toHaveLength(expected));
    }
    const second = vi.mocked(api.log).mock.calls[1]![1];
    expect(second).toMatchObject({ skip: 1, tips: ["t1"], scope: "all" });
    const before = vi.mocked(api.log).mock.calls.length;
    await act(async () => {
      invalidateLog(client, "/repo");
    });
    await waitFor(() => expect(vi.mocked(api.log).mock.calls.length).toBe(before + 1));
    expect(result.current.data?.pages).toHaveLength(1);
  });
});
