import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: { executeAction: vi.fn(), previewAction: vi.fn() },
}));
vi.mock("../../lib/copy", () => ({ copyText: vi.fn(async () => true) }));

import { AppProviders, createQueryClient } from "../../App";
import { ApiError, api, type ChangedFile } from "../../lib/api";
import { copyText } from "../../lib/copy";
import { useToastStore } from "../../ui/toastStore";
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
  it("줄은 위 파일 이름·아래 폴더, 상태 글자는 고침(D-143)", () => {
    renderList([file("docs/plans/010-settings-github-more.md", false), file("README.md", false, { change: "untracked" })]);
    const row = screen.getByRole("button", { name: /010-settings-github-more\.md/ });
    const [names, kind] = [...row.children];
    expect(names?.children[0]?.textContent).toBe("010-settings-github-more.md");
    expect(names?.children[1]?.textContent).toBe("docs/plans");
    expect(kind?.textContent).toBe("고침");
    expect(names).toHaveAttribute("title", "docs/plans/010-settings-github-more.md");
    // 최상위 파일은 폴더 줄이 비어 있다(높이는 같다).
    const top = screen.getByRole("button", { name: /README\.md/ });
    expect(top.children[0]?.children[1]?.textContent).toBe("");
    expect(top.children[1]?.textContent).toBe("새 파일");
  });

  it("이름 바꾼 파일은 아래 줄에 원래 이름", () => {
    renderList([file("새/이름.txt", false, { change: "renamed", origPath: "옛/이름.txt" })]);
    expect(screen.getByText("옛/이름.txt에서 이름 바꿈")).toBeInTheDocument();
  });

  it("넣은 파일이 0개면 모두 켠 모양이고 하나를 끄면 나머지를 넣는다(D-142)", async () => {
    const user = userEvent.setup();
    renderList([file("a.txt", false), file("b.txt", false), file("c.txt", false)]);
    for (const box of screen.getAllByRole("checkbox")) expect(box).toBeChecked();
    const head = screen.getByRole("checkbox", { name: /고친 파일 3개$/ });
    expect(head).toHaveProperty("indeterminate", false);
    await user.click(screen.getByRole("checkbox", { name: "이번 버전에서 빼기: b.txt" }));
    expect(api.executeAction).toHaveBeenCalledTimes(1);
    expect(api.executeAction).toHaveBeenCalledWith("/repo", { id: "stage", input: { paths: ["a.txt", "c.txt"] } });
  });

  it("파일 하나뿐이면 그 파일을 넣어 확정하고, 머리 체크는 모두 넣는다", async () => {
    const user = userEvent.setup();
    const { unmount } = renderList([file("한 개.txt", false)]);
    await user.click(screen.getByRole("checkbox", { name: "이번 버전에서 빼기: 한 개.txt" }));
    expect(api.executeAction).toHaveBeenLastCalledWith("/repo", { id: "stage", input: { paths: ["한 개.txt"] } });
    unmount();
    renderList([file("a", false), file("b", false)]);
    await user.click(screen.getByRole("checkbox", { name: /고친 파일 2개$/ }));
    expect(api.executeAction).toHaveBeenLastCalledWith("/repo", { id: "stage", input: { paths: ["a", "b"] } });
  });

  it("오른쪽 클릭 메뉴: 경로 복사와 이 파일 고친 것 버리기(R13)", async () => {
    const user = userEvent.setup();
    vi.mocked(api.previewAction).mockResolvedValue({ token: "t1" });
    renderList([file("docs/한글.md", false), file("b.md", false)]);
    fireEvent.contextMenu(screen.getByRole("button", { name: /한글\.md/ }));
    await user.click(await screen.findByRole("menuitem", { name: "경로 복사" }));
    expect(copyText).toHaveBeenCalledWith("docs/한글.md");
    await waitFor(() => expect(useToastStore.getState().toast?.text).toBe("복사했어요"));
    fireEvent.contextMenu(screen.getByRole("button", { name: /한글\.md/ }));
    await user.click(await screen.findByRole("menuitem", { name: "이 파일 고친 것 버리기" }));
    expect(api.previewAction).toHaveBeenCalledWith("/repo", { id: "discard", input: { paths: ["docs/한글.md"] } });
    // 되돌릴 수 없어 확인을 거친다(D-043).
    expect(await screen.findByRole("dialog")).toHaveTextContent("한글.md");
    expect(api.executeAction).not.toHaveBeenCalled();
  });

  it("메뉴 키·Shift+F10으로도 연다", async () => {
    renderList([file("a.txt", false)]);
    const row = screen.getByRole("button", { name: /a\.txt/ });
    fireEvent.keyDown(row, { key: "F10", shiftKey: true });
    expect(await screen.findByRole("menuitem", { name: "경로 복사" })).toBeInTheDocument();
  });

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
    const all = screen.getByRole("checkbox", { name: /^이번 버전에 넣기: 고친 파일 \d+개$/ });
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
    const all = screen.getByRole("checkbox", { name: /^이번 버전에서 빼기: 고친 파일 \d+개$/ });
    expect(all).toBeChecked();
    await user.click(all);
    expect(api.executeAction).toHaveBeenCalledWith("/repo", {
      id: "unstage",
      input: { paths: ["a", "b"] },
    });
  });

  it("실패하면 이유를 보여준다", async () => {
    const user = userEvent.setup();
    renderList([file("a", false), file("b", true)]);
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

  it("↑↓로 비교할 파일을 옮긴다", async () => {
    const user = userEvent.setup();
    renderList([file("a.txt", false), file("b.txt", false), file("c.txt", false)]);
    act(() => useRepoStore.setState({ selectedPath: "a.txt" }));
    document.getElementById("changed-files")!.focus();
    await user.keyboard("{ArrowDown}{ArrowDown}");
    expect(useRepoStore.getState().selectedPath).toBe("c.txt");
    await user.keyboard("{ArrowDown}");
    expect(useRepoStore.getState().selectedPath).toBe("c.txt");
    await user.keyboard("{ArrowUp}");
    expect(useRepoStore.getState().selectedPath).toBe("b.txt");
  });
});
