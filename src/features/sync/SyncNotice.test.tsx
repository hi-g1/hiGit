import { act, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: { executeAction: vi.fn() },
}));

import { AppProviders, createQueryClient } from "../../App";
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
});
