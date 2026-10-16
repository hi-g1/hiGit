import { act, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: { executeAction: vi.fn(), settings: vi.fn(async () => ({ ...DEFAULTS })) },
}));

import { AppProviders, createQueryClient } from "../../App";
import { DEFAULT_SETTINGS as DEFAULTS } from "../../lib/display";
import { useForcePushStore } from "../force-push/forcePushStore";
import { ApiError, api } from "../../lib/api";
import { SyncNotice } from "./SyncNotice";
import { useSyncStore } from "./syncStore";

function show() {
  render(
    <AppProviders client={createQueryClient()}>
      <SyncNotice repo="/repo" />
    </AppProviders>,
  );
}

afterEach(() => {
  vi.clearAllMocks();
  act(() => useSyncStore.setState({ error: null }));
});

describe("동기화 안내 줄", () => {
  it("멈춰서 그만뒀으면 안내와 [동기화]", async () => {
    vi.mocked(api.executeAction).mockReturnValue(new Promise(() => {}));
    act(() => useSyncStore.setState({ error: new ApiError({ code: "stalled", detail: null }) }));
    show();
    expect(screen.getByRole("alert")).toHaveTextContent("진행이 멈춰 그만뒀어요.");
    await userEvent.click(screen.getByRole("button", { name: "동기화" }));
    expect(api.executeAction).toHaveBeenCalledWith("/repo", { id: "sync", input: { locale: "ko" } });
    // 다시 시작하면 안내 줄을 지운다.
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("다른 오류는 안내만", () => {
    act(() => useSyncStore.setState({ error: new ApiError({ code: "pushRejected", detail: null }) }));
    show();
    expect(screen.getByRole("alert")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "동기화" })).not.toBeInTheDocument();
  });

  it("올린 기록을 바꿔 멈췄으면 [그래도 합치기]가 합치기를 허락해 다시 한다(D-129)", async () => {
    vi.mocked(api.executeAction).mockReturnValue(new Promise(() => {}));
    act(() => useSyncStore.setState({ error: new ApiError({ code: "rewrotePublished", detail: null }) }));
    show();
    expect(screen.getByRole("alert")).toHaveTextContent("올린 버전을 바꿔서 원격 저장소와 갈라졌어요.");
    // 설정이 꺼져 있으면 덮어쓰며 올리기 단추는 없다.
    expect(screen.queryByRole("button", { name: "덮어쓰며 올리기" })).toBeNull();
    await userEvent.click(screen.getByRole("button", { name: "그래도 합치기" }));
    expect(api.executeAction).toHaveBeenCalledWith("/repo", { id: "sync", input: { locale: "ko", mergeAnyway: true } });
  });

  it("설정이 켜져 있으면 덮어쓰며 올리기 창을 연다", async () => {
    vi.mocked(api.settings).mockResolvedValue({ ...DEFAULTS, allowPublishedRewrite: true });
    act(() => useSyncStore.setState({ error: new ApiError({ code: "rewrotePublished", detail: null }) }));
    show();
    await userEvent.click(await screen.findByRole("button", { name: "덮어쓰며 올리기" }));
    expect(useForcePushStore.getState().open).toBe(true);
    act(() => useForcePushStore.setState({ open: false }));
  });
});
