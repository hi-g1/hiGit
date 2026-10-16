import { render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: { diff: vi.fn() },
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

afterEach(() => vi.clearAllMocks());

describe("DiffView", () => {
  it("줄 번호, 기호, 요약을 보여준다", async () => {
    show({
      binary: false,
      truncated: false,
      link: null,
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
    expect(screen.getByRole("heading", { name: "src/한글 파일.txt" })).toBeInTheDocument();
    expect(api.diff).toHaveBeenCalledWith("/repo", request);
  });

  it("이진 파일은 내용 대신 안내", async () => {
    show({ binary: true, truncated: false, hunks: [], link: null });
    expect(await screen.findByText("이진 파일이라 내용을 비교할 수 없어요.")).toBeInTheDocument();
  });

  it("잘린 큰 파일과 차이 없음 안내", async () => {
    show({
      binary: false,
      truncated: true,
      link: null,
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
    show({ binary: false, truncated: false, hunks: [], link: null });
    expect(await screen.findByText("바뀐 줄이 없어요.")).toBeInTheDocument();
  });

  it("심볼릭 링크는 내용 대신 가리키는 곳", async () => {
    show({ binary: false, truncated: false, hunks: [], link: "../밖/비밀.txt" });
    expect(await screen.findByText("링크: ../밖/비밀.txt")).toBeInTheDocument();
    expect(screen.queryByText("바뀐 줄이 없어요.")).toBeNull();
  });
});
