import { act, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: { pendingWork: vi.fn(), previewAction: vi.fn(), executeAction: vi.fn() },
}));

import { AppProviders, createQueryClient } from "../../App";
import { ApiError, api, type PendingWorkView } from "../../lib/api";
import { useToastStore } from "../../ui/toastStore";
import { SyncNotice } from "../sync/SyncNotice";
import { useSyncStore } from "../sync/syncStore";
import { PendingNotice } from "./PendingNotice";
import { usePendingStore } from "./pendingStore";

const view = (over: Partial<PendingWorkView> = {}): PendingWorkView => ({
  staleLocks: [],
  leftoverParked: 0,
  otherOperation: null,
  ...over,
});

function show(pending: PendingWorkView, otherConflicts = false) {
  vi.mocked(api.pendingWork).mockResolvedValue(pending);
  return render(
    <AppProviders client={createQueryClient()}>
      <PendingNotice repo="/repo" otherConflicts={otherConflicts} />
    </AppProviders>,
  );
}

afterEach(() => {
  vi.clearAllMocks();
  act(() => {
    usePendingStore.setState({ dismissed: new Set() });
    useToastStore.setState({ toasts: [], toast: null });
    useSyncStore.setState({ error: null });
  });
});

describe("끝나지 않은 작업 알림", () => {
  it("알릴 것이 없으면 아무것도 그리지 않는다", async () => {
    show(view());
    await vi.waitFor(() => expect(api.pendingWork).toHaveBeenCalled());
    expect(screen.queryByRole("button", { name: "닫기" })).not.toBeInTheDocument();
  });

  it("줄마다 문장과 단추", async () => {
    show(view({ staleLocks: ["index.lock"], leftoverParked: 2, otherOperation: "rebase" }));
    expect(await screen.findByText("멈춘 작업 파일 1개")).toBeInTheDocument();
    expect(screen.getByText("남은 맡겨 둔 파일 2건")).toBeInTheDocument();
    const other = screen.getAllByRole("status")[2]!;
    expect(other).toHaveTextContent("리베이스");
    expect(other).toHaveTextContent("다른 도구에서 시작함");
    expect(within(other).queryByRole("button", { name: "정리하기" })).not.toBeInTheDocument();
  });

  it("이름 없는 다른 도구의 충돌은 일반 문구", async () => {
    show(view(), true);
    expect(await screen.findByText("다른 도구에서 시작한 작업이 멈춰 있어요.")).toBeInTheDocument();
  });

  it("정리하기는 지울 파일을 보여 주는 확인 뒤에만 실행한다", async () => {
    vi.mocked(api.previewAction).mockResolvedValue({ token: "t1" });
    vi.mocked(api.executeAction).mockResolvedValue({ kind: "locksCleared", count: 2 });
    show(view({ staleLocks: ["index.lock", "refs/heads/main.lock"] }));
    await userEvent.click(await screen.findByRole("button", { name: "정리하기" }));
    expect(api.previewAction).toHaveBeenCalledWith("/repo", { id: "clear-lock" });
    const dialog = await screen.findByRole("dialog", { name: "멈춘 작업 파일 2개를 지울까요?" });
    expect(dialog).toHaveTextContent("index.lock · refs/heads/main.lock");
    // Enter가 지우기가 되지 않게 닫기에 포커스가 있다.
    expect(within(dialog).getByRole("button", { name: "닫기" })).toHaveFocus();
    expect(api.executeAction).not.toHaveBeenCalled();
    await userEvent.click(within(dialog).getByRole("button", { name: "정리하기" }));
    expect(api.executeAction).toHaveBeenCalledWith("/repo", { id: "clear-lock" }, "t1");
    await vi.waitFor(() => expect(useToastStore.getState().toast?.text).toBe("멈춘 작업 파일 2개를 지웠어요"));
  });

  it("닫기를 누르면 확인 대화상자가 닫히고 실행하지 않는다", async () => {
    vi.mocked(api.previewAction).mockResolvedValue({ token: "t1" });
    show(view({ staleLocks: ["index.lock"] }));
    await userEvent.click(await screen.findByRole("button", { name: "정리하기" }));
    const dialog = await screen.findByRole("dialog");
    await userEvent.click(within(dialog).getByRole("button", { name: "닫기" }));
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    expect(api.executeAction).not.toHaveBeenCalled();
  });

  it("복구하기", async () => {
    vi.mocked(api.executeAction).mockResolvedValue({ kind: "parkedRecovered", count: 2 });
    show(view({ leftoverParked: 2 }));
    await userEvent.click(await screen.findByRole("button", { name: "복구하기" }));
    expect(api.executeAction).toHaveBeenCalledWith("/repo", { id: "recover-parked" });
    await vi.waitFor(() => expect(useToastStore.getState().toast?.text).toBe("맡겨 둔 파일 2건을 복구했어요"));
  });

  it("닫은 줄은 다시 보이지 않는다", async () => {
    show(view({ leftoverParked: 1, otherOperation: "revert" }));
    const line = (await screen.findByText("남은 맡겨 둔 파일 1건")).closest("[role=status]") as HTMLElement;
    await userEvent.click(within(line).getByRole("button", { name: "닫기" }));
    expect(screen.queryByText("남은 맡겨 둔 파일 1건")).not.toBeInTheDocument();
    expect(screen.getByRole("status")).toHaveTextContent("되돌리기");
  });

  it("잠금 오류 안내에는 버려진 잠금이 있을 때만 정리하기", async () => {
    vi.mocked(api.pendingWork).mockResolvedValue(view({ staleLocks: ["index.lock"] }));
    act(() => useSyncStore.setState({ error: new ApiError({ code: "indexLocked", detail: null }) }));
    render(
      <AppProviders client={createQueryClient()}>
        <SyncNotice repo="/repo" />
      </AppProviders>,
    );
    expect(screen.getByRole("alert")).toHaveTextContent("다른 Git 작업이 진행 중이에요.");
    expect(await within(screen.getByRole("alert")).findByRole("button", { name: "정리하기" })).toBeInTheDocument();
  });
});
