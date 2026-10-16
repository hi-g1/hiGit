import { QueryClient } from "@tanstack/react-query";
import { fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("../../../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../../../lib/api")>()),
  api: { commitDetail: vi.fn(), diff: vi.fn() },
}));

import { AppProviders, createQueryClient } from "../../../../App";
import { api, type CommitDetailView } from "../../../../lib/api";
import { ko } from "../../../../messages/ko";
import type { StationInfo } from "../info";
import type { MenuCtx } from "../items";
import { FilesDialog } from "./FilesDialog";

const oid = "f".repeat(40);
const info = (over: Partial<StationInfo> = {}): StationInfo => ({
  oid,
  shortOid: "fffffff",
  summary: "버튼 고침",
  author: "민지",
  time: 1_700_000_000,
  parents: ["e".repeat(40)],
  isMerge: false,
  isRoot: false,
  isHead: false,
  onHeadLine: true,
  isAncestorOfHead: true,
  branches: [],
  tags: [],
  unpushed: false,
  ...over,
});
const detail = (over: Partial<CommitDetailView> = {}): CommitDetailView => ({
  oid,
  shortOid: "fffffff",
  message: "버튼 고침",
  author: "민지",
  time: 1_700_000_000,
  parents: [],
  files: [
    { path: "src/새 파일.ts", origPath: null, change: "added" },
    { path: "src/화면.tsx", origPath: null, change: "modified" },
    { path: "옛.txt", origPath: null, change: "deleted" },
  ],
  moreFiles: 0,
  ...over,
});

function open(i = info()) {
  vi.mocked(api.diff).mockResolvedValue({ binary: false, truncated: false, hunks: [], link: null, newLineCount: null });
  const ctx = { repo: "/r", m: ko, queryClient: new QueryClient() } as unknown as MenuCtx;
  render(
    <AppProviders client={createQueryClient()}>
      <FilesDialog info={i} ctx={ctx} onClose={vi.fn()} />
    </AppProviders>,
  );
}

afterEach(() => vi.clearAllMocks());

describe("이때 파일 보기", () => {
  it("파일 목록과 상태 글자, 고른 파일의 그 버전 안 비교", async () => {
    vi.mocked(api.commitDetail).mockResolvedValue(detail());
    open();
    const list = await screen.findByRole("listbox", { name: ko.filesAtVersion });
    const rows = within(list).getAllByRole("option");
    expect(rows.map((r) => r.textContent)).toEqual(["+src/새 파일.ts", "~src/화면.tsx", "−옛.txt"]);
    expect(rows[0]).toHaveAttribute("aria-selected", "true");
    await vi.waitFor(() =>
      expect(api.diff).toHaveBeenCalledWith("/r", { path: "src/새 파일.ts", origPath: null, untracked: false, commit: oid }),
    );
    // ↓로 다음 파일.
    fireEvent.keyDown(list, { key: "ArrowDown" });
    expect(within(list).getAllByRole("option")[1]).toHaveAttribute("aria-selected", "true");
    await vi.waitFor(() =>
      expect(api.diff).toHaveBeenCalledWith("/r", { path: "src/화면.tsx", origPath: null, untracked: false, commit: oid }),
    );
  });

  it("합친 버전 머리 줄, 빈 버전, 500개 넘는 수", async () => {
    vi.mocked(api.commitDetail).mockResolvedValue(detail({ moreFiles: 7 }));
    open(info({ isMerge: true, parents: ["a".repeat(40), "b".repeat(40)] }));
    expect(await screen.findByText(ko.comparedBeforeMerge)).toBeInTheDocument();
    expect(screen.getByText(ko.moreFilesInVersion(7))).toBeInTheDocument();
  });

  it("바뀐 파일이 없는 버전", async () => {
    vi.mocked(api.commitDetail).mockResolvedValue(detail({ files: [] }));
    open();
    expect(await screen.findByText(ko.noFilesInVersion)).toBeInTheDocument();
  });
});
