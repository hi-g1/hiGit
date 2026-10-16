import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { ActionInput, ActionResult, PrDraftView, PrListView } from "../../lib/api";

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: {
    prDraft: vi.fn(),
    prList: vi.fn(),
    executeAction: vi.fn(),
    cancelAction: vi.fn(async () => null),
    openHostingPage: vi.fn(async () => undefined),
  },
}));

import { AppProviders, createQueryClient } from "../../App";
import { api } from "../../lib/api";
import { useToastStore } from "../../ui/toastStore";
import { useProgressStore } from "../progress/progressStore";
import { useSettingsStore } from "../settings/settingsStore";
import { CreatePrDialog } from "./CreatePrDialog";
import { usePrDialogStore } from "./prDialogStore";

const REPO = "/home/me/hello";
const draft = (over: Partial<PrDraftView> = {}): PrDraftView => ({
  branch: "feature/login",
  head: "feature/login",
  base: "main",
  baseOptions: ["main", "develop"],
  suggestionTitle: "로그인 화면 추가",
  ahead: 2,
  unpushed: 0,
  viaFork: false,
  ...over,
});
const noPrs: PrListView = { items: [], hasMore: false };

function show() {
  act(() => usePrDialogStore.getState().openPrDialog());
  return render(
    <AppProviders client={createQueryClient()}>
      <CreatePrDialog repo={REPO} />
    </AppProviders>,
  );
}

const createButton = () => screen.getByRole("button", { name: "검토 요청 만들기" });

beforeEach(() => {
  vi.mocked(api.prDraft).mockResolvedValue(draft());
  vi.mocked(api.prList).mockResolvedValue(noPrs);
  vi.mocked(api.executeAction).mockReset();
  vi.mocked(api.openHostingPage).mockClear();
});

afterEach(() => {
  act(() => {
    usePrDialogStore.getState().close();
    useSettingsStore.getState().close();
    useProgressStore.setState({ byRepo: {} });
    useToastStore.getState().clear();
  });
});

describe("CreatePrDialog", () => {
  it("기본값으로 채우고, 이미 올라가 있으면 검토 요청만 만든다", async () => {
    vi.mocked(api.executeAction).mockResolvedValue({ kind: "prCreated", number: 12 } as ActionResult);
    show();
    const title = await screen.findByLabelText("제목");
    expect(title).toHaveValue("로그인 화면 추가");
    expect(screen.getByText("feature/login → main")).toBeInTheDocument();
    expect(screen.getByText("main보다 앞선 버전 2개")).toBeInTheDocument();
    fireEvent.change(screen.getByRole("combobox"), { target: { value: "develop" } });
    fireEvent.click(screen.getByRole("checkbox"));
    fireEvent.change(screen.getByLabelText("설명 추가"), { target: { value: "본문" } });
    fireEvent.click(createButton());
    await waitFor(() => expect(usePrDialogStore.getState().open).toBe(false));
    expect(api.executeAction).toHaveBeenCalledTimes(1);
    expect(api.executeAction).toHaveBeenCalledWith(REPO, {
      id: "pr-create",
      input: { title: "로그인 화면 추가", body: "본문", base: "develop", draft: true },
    });
    const toast = useToastStore.getState().toast;
    expect(toast?.text).toBe("검토 요청을 만들었어요");
    expect(toast?.action?.label).toBe("열기");
    toast?.action?.onClick();
    expect(api.openHostingPage).toHaveBeenCalledWith(REPO, { kind: "pullRequest", number: 12 });
  });

  it("올리지 않은 버전이 있으면 먼저 올리고, 도는 동안 진행과 취소", async () => {
    vi.mocked(api.prDraft).mockResolvedValue(draft({ head: null, unpushed: 2 }));
    let finishPush: (r: ActionResult) => void = () => undefined;
    vi.mocked(api.executeAction).mockImplementation((_repo, input: ActionInput) =>
      input.id === "push"
        ? new Promise<ActionResult>((resolve) => (finishPush = resolve))
        : Promise.resolve({ kind: "prCreated", number: 3 } as ActionResult),
    );
    show();
    expect(await screen.findByText(/올리기하지 않은 버전 2개/)).toBeInTheDocument();
    fireEvent.click(createButton());
    expect(await screen.findByRole("status")).toHaveTextContent("올리기 중 · 0초");
    act(() => useProgressStore.getState().receive({ repo: REPO, action: "push", elapsedSecs: 5, idleSecs: 0, phase: null, done: null, total: null }));
    fireEvent.click(screen.getByRole("button", { name: "취소" }));
    expect(api.cancelAction).toHaveBeenCalledWith(REPO);
    await act(async () => finishPush({ kind: "synced", received: 0, sent: 2, merged: false, overlap: [], mergeStarted: false } as ActionResult));
    await waitFor(() => expect(api.executeAction).toHaveBeenCalledTimes(2));
    expect(vi.mocked(api.executeAction).mock.calls.map((c) => c[1].id)).toEqual(["push", "pr-create"]);
  });

  it("제목이 비거나 차이가 없으면 만들 수 없다", async () => {
    vi.mocked(api.prDraft).mockResolvedValue(draft({ ahead: 0 }));
    show();
    expect(await screen.findByText("main 브랜치와 같아요")).toBeInTheDocument();
    expect(createButton()).toBeDisabled();
    fireEvent.change(screen.getByRole("combobox"), { target: { value: "develop" } });
    expect(createButton()).toBeEnabled();
    fireEvent.change(screen.getByLabelText("제목"), { target: { value: "  " } });
    expect(createButton()).toBeDisabled();
  });

  it("이미 열린 검토 요청이 있으면 그것만 보인다", async () => {
    vi.mocked(api.prList).mockResolvedValue({
      items: [
        {
          number: 5,
          title: "있음",
          state: "open",
          draft: false,
          head: "feature/login",
          headOwner: null,
          base: "main",
          author: "mona",
          updatedAt: 0,
          isCurrentBranch: true,
        },
      ],
      hasMore: false,
    });
    show();
    expect(await screen.findByText("검토 요청 #5이 이미 있어요")).toBeInTheDocument();
    expect(createButton()).toBeDisabled();
    fireEvent.click(screen.getByRole("button", { name: "GitHub에서 열기" }));
    expect(api.openHostingPage).toHaveBeenCalledWith(REPO, { kind: "pullRequest", number: 5 });
    expect(api.prList).toHaveBeenCalledWith(REPO, { state: "all", branch: "feature/login", page: 1 });
  });

  it("오류는 대화상자 안에, 연결이 끊겼으면 다시 연결", async () => {
    vi.mocked(api.executeAction).mockRejectedValueOnce({ code: "prNoDifference", detail: null });
    show();
    await screen.findByLabelText("제목");
    fireEvent.click(createButton());
    expect(await screen.findByText("두 브랜치 사이에 다른 버전이 없어요.")).toBeInTheDocument();
    vi.mocked(api.executeAction).mockRejectedValueOnce({ code: "githubExpired", detail: null });
    fireEvent.click(createButton());
    fireEvent.click(await screen.findByRole("button", { name: "다시 연결" }));
    expect(useSettingsStore.getState().connect).toBe(true);
  });

  it("Ctrl+Enter로 만들고, 한글 조합 중에는 무시한다", async () => {
    vi.mocked(api.executeAction).mockResolvedValue({ kind: "prCreated", number: 1 } as ActionResult);
    show();
    const title = await screen.findByLabelText("제목");
    fireEvent.keyDown(title, { key: "Enter", code: "Enter", ctrlKey: true, isComposing: true });
    expect(api.executeAction).not.toHaveBeenCalled();
    fireEvent.keyDown(title, { key: "Enter", code: "Enter", ctrlKey: true });
    await waitFor(() => expect(api.executeAction).toHaveBeenCalledTimes(1));
  });
});
