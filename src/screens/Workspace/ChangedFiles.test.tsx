import { act, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: { executeAction: vi.fn() },
}));

import { AppProviders, createQueryClient } from "../../App";
import { ApiError, api, type ChangedFile } from "../../lib/api";
import { useRepoStore } from "../../state/repoStore";
import { ChangedFiles } from "./ChangedFiles";

const file = (path: string, staged: boolean, extra: Partial<ChangedFile> = {}): ChangedFile => ({
  path,
  origPath: null,
  change: "modified",
  staged,
  ...extra,
});

function renderList(files: ChangedFile[]) {
  vi.mocked(api.executeAction).mockResolvedValue({ kind: "staged", count: 1 });
  return render(
    <AppProviders client={createQueryClient()}>
      <ChangedFiles folder="/repo" files={files} />
    </AppProviders>,
  );
}

afterEach(() => {
  vi.clearAllMocks();
  act(() => useRepoStore.setState({ selectedPath: null }));
});

describe("ChangedFiles", () => {
  it("체크하면 이번 버전에 넣고, 해제하면 뺀다", async () => {
    const user = userEvent.setup();
    renderList([file("한글 파일.txt", false), file("b.md", true)]);

    await user.click(screen.getByRole("checkbox", { name: "이번 버전에 넣기: 한글 파일.txt" }));
    expect(api.executeAction).toHaveBeenCalledWith("/repo", {
      id: "stage",
      input: { paths: ["한글 파일.txt"] },
    });

    await user.click(screen.getByRole("checkbox", { name: "이번 버전에서 빼기: b.md" }));
    expect(api.executeAction).toHaveBeenLastCalledWith("/repo", {
      id: "unstage",
      input: { paths: ["b.md"] },
    });
  });

  it("이름 바꾼 파일을 빼면 원래 경로도 함께 되돌린다", async () => {
    const user = userEvent.setup();
    renderList([file("새 이름.md", true, { origPath: "옛 이름.md", change: "renamed" })]);
    await user.click(screen.getByRole("checkbox", { name: "이번 버전에서 빼기: 새 이름.md" }));
    expect(api.executeAction).toHaveBeenCalledWith("/repo", {
      id: "unstage",
      input: { paths: ["새 이름.md", "옛 이름.md"] },
    });
  });

  it("모두 체크 상자는 세 상태이고, 일부만 넣었으면 나머지를 넣는다", async () => {
    const user = userEvent.setup();
    renderList([file("a", true), file("b", false), file("c", false)]);
    const all = screen.getByRole("checkbox", { name: "이번 버전에 넣기: 모든 파일" });
    expect(all).toHaveProperty("indeterminate", true);
    await user.click(all);
    expect(api.executeAction).toHaveBeenCalledWith("/repo", {
      id: "stage",
      input: { paths: ["b", "c"] },
    });
  });

  it("모두 넣었으면 모두 뺀다", async () => {
    const user = userEvent.setup();
    renderList([file("a", true), file("b", true)]);
    const all = screen.getByRole("checkbox", { name: "이번 버전에서 빼기: 모든 파일" });
    expect(all).toBeChecked();
    await user.click(all);
    expect(api.executeAction).toHaveBeenCalledWith("/repo", {
      id: "unstage",
      input: { paths: ["a", "b"] },
    });
  });

  it("실패하면 이유를 보여준다", async () => {
    const user = userEvent.setup();
    renderList([file("a", false)]);
    vi.mocked(api.executeAction).mockRejectedValue(new ApiError({ code: "busy", detail: null }));
    await user.click(screen.getByRole("checkbox", { name: "이번 버전에 넣기: a" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("다른 작업이 끝날 때까지 기다려 주세요.");
  });

  it("행을 누르면 비교할 파일로 고른다", async () => {
    const user = userEvent.setup();
    renderList([file("a.txt", false), file("b.txt", false)]);
    await user.click(screen.getByRole("button", { name: /b\.txt/ }));
    expect(useRepoStore.getState().selectedPath).toBe("b.txt");
    expect(screen.getByRole("button", { name: /b\.txt/ })).toHaveAttribute("aria-pressed", "true");
    expect(api.executeAction).not.toHaveBeenCalled();
  });
});
