import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn(async () => () => undefined) }));
vi.mock("../../lib/dialog", () => ({ pickFolder: vi.fn(async () => "/data/새 폴더") }));
vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: {
    appStatus: vi.fn(async () => ({ gitVersion: "2.50.1", gitSecurity: "ok", gitPackageVersion: null, gitPath: "/usr/bin/git", undoKey: "saved", undoKeyDetail: null })),
    cloneDefaults: vi.fn(async () => ({ parent: "/home/me" })),
    githubStatus: vi.fn(async () => ({ configured: true, connected: false, login: null, remembered: false, workflow: false, email: false })),
    executeAction: vi.fn(),
    cancelAction: vi.fn(async () => null),
    openProject: vi.fn(async () => ({ kind: "opened", repo: { path: "/home/me/hello", name: "hello" } })),
  },
}));

import { AppProviders, createQueryClient } from "../../App";
import { api, type ActionResult } from "../../lib/api";
import { useProgressStore } from "../progress/progressStore";
import { useSettingsStore } from "../settings/settingsStore";
import { CloneDialog } from "./CloneDialog";
import { useCloneStore } from "./cloneStore";

function show() {
  act(() => useCloneStore.getState().openClone());
  return render(
    <AppProviders client={createQueryClient()}>
      <CloneDialog />
    </AppProviders>,
  );
}

const urlField = () => screen.getByLabelText("주소");
const nameField = () => screen.getByLabelText("폴더 이름");
const runButton = () => screen.getByRole("button", { name: "복제" });

beforeEach(() => {
  vi.mocked(api.executeAction).mockReset();
  vi.mocked(api.cancelAction).mockClear();
  vi.mocked(api.openProject).mockClear();
});

afterEach(() => {
  act(() => {
    useCloneStore.getState().close();
    useSettingsStore.getState().close();
    useProgressStore.setState({ byRepo: {} });
  });
});

describe("CloneDialog", () => {
  it("주소로 이름을 채우고 부모 폴더 아래로 복제한 뒤 연다", async () => {
    vi.mocked(api.executeAction).mockResolvedValue({
      kind: "repoCreated",
      repo: { path: "/home/me/hello", name: "hello" },
    } as ActionResult);
    show();
    expect(runButton()).toBeDisabled();
    fireEvent.change(urlField(), { target: { value: "https://github.com/octo/hello.git" } });
    expect(nameField()).toHaveValue("hello");
    expect(await screen.findByText("/home/me/hello")).toBeInTheDocument();
    fireEvent.click(runButton());
    await waitFor(() => expect(api.openProject).toHaveBeenCalledWith("/home/me/hello", "picked"));
    expect(api.executeAction).toHaveBeenCalledWith("/home/me", {
      id: "clone",
      input: { url: "https://github.com/octo/hello.git", name: "hello", acceptOldGit: false },
    });
    expect(useCloneStore.getState().open).toBe(false);
  });

  it("고친 이름은 주소가 바뀌어도 그대로, 저장할 곳은 바꿀 수 있다", async () => {
    show();
    fireEvent.change(urlField(), { target: { value: "https://github.com/octo/a" } });
    fireEvent.change(nameField(), { target: { value: "내 이름" } });
    fireEvent.change(urlField(), { target: { value: "https://github.com/octo/b" } });
    expect(nameField()).toHaveValue("내 이름");
    fireEvent.click(screen.getByRole("button", { name: "바꾸기" }));
    expect(await screen.findByText("/data/새 폴더/내 이름")).toBeInTheDocument();
  });

  it("붙여넣은 주소의 앞뒤 공백을 뗀다", () => {
    show();
    fireEvent.paste(urlField(), { clipboardData: { getData: () => "  git@github.com:octo/hello.git \n" } });
    expect(urlField()).toHaveValue("git@github.com:octo/hello.git");
    expect(nameField()).toHaveValue("hello");
  });

  it("한글 조합 중 Enter는 실행하지 않는다", () => {
    show();
    fireEvent.change(urlField(), { target: { value: "https://github.com/octo/hello" } });
    fireEvent.keyDown(nameField(), { key: "Enter", isComposing: true });
    expect(api.executeAction).not.toHaveBeenCalled();
  });

  it("진행을 보이고 3초 뒤 취소, 도는 동안 닫히지 않는다", async () => {
    let finish: (value: ActionResult) => void = () => undefined;
    vi.mocked(api.executeAction).mockImplementation(
      () => new Promise<ActionResult>((resolve) => (finish = resolve)),
    );
    show();
    fireEvent.change(urlField(), { target: { value: "https://github.com/octo/hello" } });
    await screen.findByText("/home/me/hello");
    fireEvent.click(runButton());
    expect(await screen.findByRole("status")).toHaveTextContent("복제 중 · 0초");
    expect(screen.queryByRole("button", { name: "취소" })).toBeNull();
    act(() =>
      useProgressStore.getState().receive({ repo: "/home/me/hello", action: "clone", elapsedSecs: 4, idleSecs: 0, phase: null, done: null, total: null }),
    );
    expect(screen.getByRole("status")).toHaveTextContent("복제 중 · 4초");
    fireEvent.click(screen.getByRole("button", { name: "취소" }));
    expect(api.cancelAction).toHaveBeenCalledWith("/home/me/hello");
    fireEvent.keyDown(urlField(), { key: "Escape" });
    expect(useCloneStore.getState().open).toBe(true);
    expect(urlField()).toBeDisabled();
    await act(async () => finish({ kind: "repoCreated", repo: { path: "/home/me/hello", name: "hello" } } as ActionResult));
  });

  it("오류는 대화상자 안에, 로그인 실패면 GitHub 연결 단추", async () => {
    vi.mocked(api.executeAction).mockRejectedValueOnce({ code: "cloneTargetExists", detail: null });
    show();
    fireEvent.change(urlField(), { target: { value: "https://github.com/octo/hello" } });
    await screen.findByText("/home/me/hello");
    fireEvent.click(runButton());
    expect(await screen.findByText("같은 이름의 폴더가 이미 있어요. 다른 이름을 써 주세요.")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "GitHub 연결" })).toBeNull();

    vi.mocked(api.executeAction).mockRejectedValueOnce({ code: "httpAuthFailed", detail: null });
    fireEvent.click(runButton());
    fireEvent.click(await screen.findByRole("button", { name: "GitHub 연결" }));
    expect(useSettingsStore.getState()).toMatchObject({ open: true, connect: true });
    expect(useCloneStore.getState().open).toBe(false);
  });

  it("Git이 오래됐으면 확인해야 복제할 수 있다", async () => {
    vi.mocked(api.appStatus).mockResolvedValueOnce({ gitVersion: "2.34.1", gitSecurity: "outdated", gitPackageVersion: null, gitPath: "/usr/bin/git", undoKey: "saved", undoKeyDetail: null });
    show();
    fireEvent.change(urlField(), { target: { value: "https://github.com/octo/hello" } });
    const check = await screen.findByLabelText("Git이 오래됐어요 · 그래도 복제");
    expect(runButton()).toBeDisabled();
    fireEvent.click(check);
    expect(runButton()).toBeEnabled();
  });

  it("열린 동안 키가 뒤로 가지 않는다", () => {
    const outside = vi.fn();
    window.addEventListener("keydown", outside);
    show();
    fireEvent.keyDown(urlField(), { key: "h", code: "KeyH", ctrlKey: true });
    expect(outside).not.toHaveBeenCalled();
    window.removeEventListener("keydown", outside);
  });
});
