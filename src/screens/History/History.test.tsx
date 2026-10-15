import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: { log: vi.fn(), logStats: vi.fn(async () => []) },
}));

import { AppProviders, createQueryClient } from "../../App";
import { api, type CommitRow, type LogQuery } from "../../lib/api";
import { useHistoryStore } from "../../state/historyStore";
import { logPage } from "../../test/logPage";
import { History } from "./History";

const oid = (i: number) => `${i}`.padStart(40, "0");
const row = (i: number, last: number): CommitRow => ({
  oid: oid(i),
  shortOid: oid(i).slice(0, 7),
  parents: i < last ? [oid(i + 1)] : [],
  summary: `버전 ${i}`,
  authorName: "지원",
  authorEmail: "j@example.com",
  authorTime: 1_700_000_000 - i * 60,
});

/** 버전 `total`개 기록을 `size`개씩. 지금 위치는 `head`. */
function serve(total: number, size: number, head = 0) {
  vi.mocked(api.log).mockImplementation(async (_repo: string, q: LogQuery) => {
    const commits = [];
    for (let i = q.skip; i < Math.min(total, q.skip + size); i++) commits.push(row(i, total - 1));
    return logPage({
      commits,
      hasMore: q.skip + size < total,
      tips: q.tips ?? ["t"],
      head: q.tips ? null : oid(head),
      currentBranch: q.tips ? null : "main",
      defaultBranch: q.tips ? null : "main",
      defaultTip: q.tips ? null : oid(0),
      labels: q.skip === 0 ? [{ oid: oid(0), kind: "branch", name: "main" }] : [],
    });
  });
}

function show() {
  return render(
    <AppProviders client={createQueryClient()}>
      <History repo="/repo" />
    </AppProviders>,
  );
}

afterEach(() => {
  vi.clearAllMocks();
  act(() => useHistoryStore.setState({ selectedId: null, zoom: "all", scope: "all", expanded: new Set() }));
});

describe("버전 기록 노선도 화면", () => {
  it("노선도 목록과 머리 줄", async () => {
    serve(5, 200);
    show();
    expect(await screen.findByRole("listbox", { name: "버전 기록" })).toBeInTheDocument();
    expect(screen.getAllByRole("option")).toHaveLength(5);
    expect(screen.getByRole("button", { name: "모든 브랜치" })).toHaveAttribute("aria-pressed", "true");
    expect(screen.getByRole("status")).toHaveTextContent("모든 버전 · 5개");
  });

  it("확대·축소는 다시 묻지 않고 접는다", async () => {
    serve(8, 200);
    show();
    await screen.findByRole("listbox");
    const calls = vi.mocked(api.log).mock.calls.length;
    await userEvent.click(screen.getByRole("button", { name: "줄이기" }));
    expect(screen.getByRole("status")).toHaveTextContent("주요 버전");
    expect(screen.getByText(/버전 \d+개 · 펼치기/)).toBeInTheDocument();
    expect(vi.mocked(api.log).mock.calls.length).toBe(calls);
    await userEvent.click(screen.getByRole("button", { name: "늘리기" }));
    expect(screen.getByRole("status")).toHaveTextContent("모든 버전");
  });

  it("+·-는 입력칸 밖에서만, Ctrl+휠도 단계를 바꾼다", async () => {
    serve(8, 200);
    const { container } = show();
    await screen.findByRole("listbox");
    fireEvent.keyDown(document.body, { code: "Minus", key: "-" });
    expect(screen.getByRole("status")).toHaveTextContent("주요 버전");
    const input = document.createElement("input");
    container.append(input);
    fireEvent.keyDown(input, { code: "Minus", key: "-" });
    expect(screen.getByRole("status")).toHaveTextContent("주요 버전");
    input.remove();
    const section = screen.getByRole("region", { name: "버전 기록" });
    fireEvent.wheel(section, { ctrlKey: true, deltaY: -120 });
    expect(screen.getByRole("status")).toHaveTextContent("모든 버전");
  });

  it("범위를 바꾸면 처음부터 지금 브랜치만", async () => {
    serve(3, 200);
    show();
    await screen.findByRole("listbox");
    await userEvent.click(screen.getByRole("button", { name: "지금 브랜치만" }));
    await waitFor(() => expect(vi.mocked(api.log).mock.calls.at(-1)![1]).toMatchObject({ skip: 0, scope: "current" }));
  });

  it("행이 모자라면 몇 쪽 더 받되 10쪽에서 멈추고, 다음 쪽은 첫 쪽의 끝점으로", async () => {
    serve(1000, 2);
    show();
    await screen.findByRole("listbox");
    await waitFor(() => expect(api.log).toHaveBeenCalledTimes(11));
    expect(vi.mocked(api.log).mock.calls[1]![1]).toMatchObject({ skip: 2, tips: ["t"] });
    await new Promise((r) => setTimeout(r, 50));
    expect(api.log).toHaveBeenCalledTimes(11);
  });

  it("지금 여기: 아직 안 받은 지금 위치를 이어 받아 고른다", async () => {
    serve(200, 20, 150);
    show();
    await screen.findByRole("listbox");
    await userEvent.click(screen.getByRole("button", { name: /지금 여기/ }));
    await waitFor(() => expect(screen.getByRole("listbox").getAttribute("aria-activedescendant")).toBe("route-row-150"));
  });
});
