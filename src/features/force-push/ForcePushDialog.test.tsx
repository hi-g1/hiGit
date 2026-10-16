import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { ForcePushPlanView } from "../../lib/api";

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: {
    settings: vi.fn(),
    forcePushPlan: vi.fn(),
    previewAction: vi.fn(),
    executeAction: vi.fn(),
    cancelAction: vi.fn(async () => null),
  },
}));

import { AppProviders, createQueryClient } from "../../App";
import { ApiError, api } from "../../lib/api";
import { DEFAULT_SETTINGS } from "../../lib/display";
import { useToastStore } from "../../ui/toastStore";
import { ForcePushDialog } from "./ForcePushDialog";
import { useForcePushStore } from "./forcePushStore";

const plan = (over: Partial<ForcePushPlanView> = {}): ForcePushPlanView => ({
  allowed: true,
  blocked: null,
  needsNameConfirm: false,
  branch: "기능",
  remote: "origin",
  remoteBranch: "기능",
  expectRemote: "a".repeat(40),
  expectHead: "b".repeat(40),
  ahead: 1,
  dropped: 7,
  droppedByOthers: 2,
  sample: [
    { shortOid: "1111111", summary: "로그인 화면", author: "동료" },
    { shortOid: "2222222", summary: "버튼 문구", author: "나" },
  ],
  rewrotePublished: true,
  ...over,
});

function show(allow = true) {
  vi.mocked(api.settings).mockResolvedValue({ ...DEFAULT_SETTINGS, allowPublishedRewrite: allow });
  act(() => useForcePushStore.setState({ open: true }));
  render(
    <AppProviders client={createQueryClient()}>
      <ForcePushDialog repo="/repo" />
    </AppProviders>,
  );
}

beforeEach(() => vi.mocked(api.forcePushPlan).mockResolvedValue(plan()));
afterEach(() => {
  vi.clearAllMocks();
  act(() => {
    useForcePushStore.setState({ open: false });
    useToastStore.setState({ toasts: [], toast: null });
  });
});

describe("덮어쓰며 올리기 확인 창(D-128)", () => {
  it("계획 값을 보이고 포커스는 닫기, Enter가 위험 버튼을 누르지 않는다", async () => {
    show();
    expect(await screen.findByText("사라지는 버전 7개")).toBeInTheDocument();
    expect(screen.getByText("올라가는 버전 1개")).toBeInTheDocument();
    expect(screen.getByText("다른 사람의 버전 2개")).toBeInTheDocument();
    expect(screen.getByText("기능 → origin/기능")).toBeInTheDocument();
    expect(screen.getByText("로그인 화면")).toBeInTheDocument();
    expect(screen.getByText("그 밖에 5개")).toBeInTheDocument();
    const close = screen.getByRole("button", { name: "닫기" });
    expect(close).toHaveFocus();
    await userEvent.keyboard("{Enter}");
    expect(api.previewAction).not.toHaveBeenCalled();
    expect(api.executeAction).not.toHaveBeenCalled();
  });

  it("미리보기 토큰을 받아 계획이 준 값 그대로 실행하고 덮어썼다고 알린다", async () => {
    vi.mocked(api.previewAction).mockResolvedValue({ token: "토큰" });
    vi.mocked(api.executeAction).mockResolvedValue({ kind: "forcePushed", dropped: 7 });
    show();
    await screen.findByText("사라지는 버전 7개");
    const danger = screen.getAllByRole("button", { name: "덮어쓰며 올리기" }).at(-1)!;
    await userEvent.click(danger);
    const input = {
      id: "force-push",
      input: { remote: "origin", branch: "기능", expectRemote: "a".repeat(40), expectHead: "b".repeat(40) },
    };
    await waitFor(() => expect(api.executeAction).toHaveBeenCalledWith("/repo", input, "토큰"));
    expect(api.previewAction).toHaveBeenCalledWith("/repo", input);
    await waitFor(() => expect(useForcePushStore.getState().open).toBe(false));
    expect(useToastStore.getState().toast?.text).toBe("덮어썼어요");
  });

  it("막혔으면 까닭 한 줄과 꺼진 위험 버튼", async () => {
    vi.mocked(api.forcePushPlan).mockResolvedValue(plan({ blocked: "otherOperation", sample: [] }));
    show();
    expect(await screen.findByText("다른 작업이 진행 중이에요")).toBeInTheDocument();
    expect(screen.getAllByRole("button", { name: "덮어쓰며 올리기" }).at(-1)).toBeDisabled();
    expect(screen.queryByText(/사라지는 버전/)).toBeNull();
  });

  it("설정이 꺼져 있으면 계획을 읽지 않고 켜라고 알린다", async () => {
    show(false);
    expect(await screen.findByText('설정 "고급"에서 "올린 버전도 바꾸기"를 켜 주세요')).toBeInTheDocument();
    expect(api.forcePushPlan).not.toHaveBeenCalled();
  });

  it("그사이 원격이 바뀌었으면 창 안에 알리고 원격 확인 단추를 준다", async () => {
    vi.mocked(api.previewAction).mockResolvedValue({ token: "토큰" });
    vi.mocked(api.executeAction).mockRejectedValueOnce(new ApiError({ code: "remoteMoved", detail: null }));
    show();
    await screen.findByText("사라지는 버전 7개");
    await userEvent.click(screen.getAllByRole("button", { name: "덮어쓰며 올리기" }).at(-1)!);
    expect(await screen.findByRole("alert")).toHaveTextContent("그사이 원격 저장소가 바뀌었어요");
    vi.mocked(api.executeAction).mockResolvedValueOnce({ kind: "fetched" });
    await userEvent.click(screen.getByRole("button", { name: "원격 확인" }));
    expect(api.executeAction).toHaveBeenLastCalledWith("/repo", { id: "fetch", input: { background: false } });
    expect(useForcePushStore.getState().open).toBe(false);
  });
});
