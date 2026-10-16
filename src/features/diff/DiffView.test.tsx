import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: { diff: vi.fn(), setSettings: vi.fn(async (p: object) => p), imagePair: vi.fn() },
}));

import { AppProviders, createQueryClient } from "../../App";
import { api, type FileDiffView } from "../../lib/api";
import { DiffView } from "./DiffView";

const request = { path: "src/한글 파일.txt", origPath: null, untracked: false };

function show(diff: FileDiffView) {
  vi.mocked(api.diff).mockResolvedValue(diff);
  render(
    <AppProviders client={createQueryClient()}>
      <DiffView repo="/repo" request={request} />
    </AppProviders>,
  );
}

afterEach(() => {
  vi.clearAllMocks();
  vi.useRealTimers();
});

const ctx = (n: number) => ({ kind: "context" as const, text: `줄 ${n}`, oldLine: n, newLine: n, noNewlineAtEnd: false });
const lines = (from: number, count: number) => Array.from({ length: count }, (_, i) => ctx(from + i));
const spaced: FileDiffView = {
  binary: false,
  truncated: false,
  link: null,
  newLineCount: null,
  hunks: [{ section: "function save()", oldStart: 13, oldLen: 3, newStart: 13, newLen: 3, lines: lines(13, 3) }],
};

describe("접힌 줄과 줄 바꿈(D-144, D-156)", () => {
  it("변경 없는 N줄 · 펼치기, 누르면 전체 문맥을 한 번 읽어 그 구간을 펼친다", async () => {
    vi.mocked(api.diff).mockImplementation(async (_repo, req) =>
      req.fullContext ? { ...spaced, hunks: [{ ...spaced.hunks[0]!, oldStart: 1, oldLen: 15, newStart: 1, newLen: 15, lines: lines(1, 15) }] } : spaced,
    );
    render(
      <AppProviders client={createQueryClient()}>
        <DiffView repo="/repo" request={request} />
      </AppProviders>,
    );
    const fold = await screen.findByRole("button", { name: "변경 없는 12줄 · 펼치기" });
    expect(screen.getByText("function save()")).toBeInTheDocument();
    expect(screen.queryByText("…")).toBeNull();
    fireEvent.click(fold);
    await waitFor(() => expect(screen.getByText("줄 1")).toBeInTheDocument());
    expect(api.diff).toHaveBeenLastCalledWith("/repo", { ...request, fullContext: true });
    expect(screen.queryByRole("button", { name: /펼치기/ })).toBeNull();
  });

  it("줄 바꿈은 기본 켬이고 끄면 저장한다", async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    show(spaced);
    const wrap = await screen.findByRole("button", { name: "줄 바꿈" });
    expect(wrap).toHaveAttribute("aria-pressed", "true");
    expect(document.querySelector("table")?.className).toMatch(/wrapOn/);
    fireEvent.click(wrap);
    await waitFor(() => expect(screen.getByRole("button", { name: "줄 바꿈" })).toHaveAttribute("aria-pressed", "false"));
    expect(document.querySelector("table")?.className).not.toMatch(/wrapOn/);
    await act(() => vi.advanceTimersByTimeAsync(300));
    expect(api.setSettings).toHaveBeenCalledWith({ diffWrap: false });
  });
});

describe("DiffView", () => {
  it("줄 번호, 기호, 요약을 보여준다", async () => {
    show({
      binary: false,
      truncated: false,
      link: null,
      newLineCount: null,
      hunks: [
        {
          section: "",
          oldStart: 1,
          oldLen: 2,
          newStart: 1,
          newLen: 2,
          lines: [
            { kind: "context", text: "가", oldLine: 1, newLine: 1, noNewlineAtEnd: false },
            { kind: "removed", text: "나", oldLine: 2, newLine: null, noNewlineAtEnd: false },
            { kind: "added", text: "나나", oldLine: null, newLine: 2, noNewlineAtEnd: true },
          ],
        },
      ],
    });
    expect(await screen.findByText("나나")).toBeInTheDocument();
    expect(screen.getByText("1줄 추가")).toBeInTheDocument();
    expect(screen.getByText("1줄 삭제")).toBeInTheDocument();
    const added = screen.getByText("나나").closest("tr")!;
    expect(added).toHaveTextContent("2+나나");
    expect(screen.getByText("파일 끝 줄바꿈 없음")).toBeInTheDocument();
    // 경로는 자르지 않고 줄을 바꾼다. title 툴팁을 쓰지 않는다(머리에는 용어 툴팁이 있다).
    expect(screen.getByRole("heading", { name: "src/한글 파일.txt" })).not.toHaveAttribute("title");
    expect(api.diff).toHaveBeenCalledWith("/repo", request);
  });

  it("이진 이미지는 이미지 비교, 너무 크면 지금 안내(D-206)", async () => {
    vi.mocked(api.imagePair).mockResolvedValue({
      before: { mime: "image/png", data: "AAAA", bytes: 3 },
      after: { mime: "image/png", data: "BBBB", bytes: 3 },
      tooLarge: false,
    });
    show({ binary: true, truncated: false, hunks: [], link: null, newLineCount: null });
    expect(await screen.findByRole("img", { name: "한글 파일.txt · 지금" })).toBeInTheDocument();
    expect(api.imagePair).toHaveBeenCalledWith("/repo", { path: "src/한글 파일.txt", origPath: null, commit: null });
  });

  it("이진 파일은 내용 대신 안내", async () => {
    vi.mocked(api.imagePair).mockResolvedValue({ before: null, after: null, tooLarge: true });
    show({ binary: true, truncated: false, hunks: [], link: null, newLineCount: null });
    expect(await screen.findByText("이진 파일이라 내용을 비교할 수 없어요.")).toBeInTheDocument();
  });

  it("잘린 큰 파일과 차이 없음 안내", async () => {
    show({
      binary: false,
      truncated: true,
      link: null,
      newLineCount: null,
      hunks: [
        {
          section: "",
          oldStart: 0,
          oldLen: 0,
          newStart: 1,
          newLen: 1,
          lines: [{ kind: "added", text: "x", oldLine: null, newLine: 1, noNewlineAtEnd: false }],
        },
      ],
    });
    expect(await screen.findByText("파일이 커서 앞부분만 보여요.")).toBeInTheDocument();
  });

  it("바뀐 줄이 없으면 그렇게 알려준다", async () => {
    show({ binary: false, truncated: false, hunks: [], link: null, newLineCount: null });
    expect(await screen.findByText("바뀐 줄이 없어요.")).toBeInTheDocument();
  });

  it("심볼릭 링크는 내용 대신 가리키는 곳", async () => {
    show({ binary: false, truncated: false, hunks: [], link: "../밖/비밀.txt", newLineCount: null });
    expect(await screen.findByText("링크: ../밖/비밀.txt")).toBeInTheDocument();
    expect(screen.queryByText("바뀐 줄이 없어요.")).toBeNull();
  });
});
