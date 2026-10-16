import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: { settings: vi.fn(), forcePushPlan: vi.fn() },
}));

import { AppProviders, createQueryClient } from "../../App";
import { api, type ForcePushPlanView } from "../../lib/api";
import { DEFAULT_SETTINGS } from "../../lib/display";
import { RewriteNotice } from "./RewriteNotice";
import { useForcePushStore } from "./forcePushStore";

const plan = (rewrotePublished: boolean): ForcePushPlanView => ({
  allowed: true,
  blocked: null,
  branch: "기능",
  remote: "origin",
  remoteBranch: "기능",
  expectRemote: "a".repeat(40),
  expectHead: "b".repeat(40),
  ahead: 1,
  dropped: 1,
  droppedByOthers: 0,
  sample: [],
  rewrotePublished,
});

function show(allow: boolean, marked: boolean) {
  vi.mocked(api.settings).mockResolvedValue({ ...DEFAULT_SETTINGS, allowPublishedRewrite: allow });
  vi.mocked(api.forcePushPlan).mockResolvedValue(plan(marked));
  render(
    <AppProviders client={createQueryClient()}>
      <RewriteNotice repo="/repo" />
    </AppProviders>,
  );
}

afterEach(() => {
  vi.clearAllMocks();
  act(() => useForcePushStore.setState({ open: false }));
});

describe("올린 버전을 바꿨어요 줄(D-129)", () => {
  it("표식이 있을 때만 보이고 덮어쓰며 올리기 창을 연다", async () => {
    show(true, true);
    expect(await screen.findByText("올린 버전을 바꿨어요")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "덮어쓰며 올리기" }));
    expect(useForcePushStore.getState().open).toBe(true);
  });

  it("표식이 없거나 설정이 꺼져 있으면 없다", async () => {
    show(true, false);
    await waitFor(() => expect(api.forcePushPlan).toHaveBeenCalled());
    expect(screen.queryByText("올린 버전을 바꿨어요")).toBeNull();
  });

  it("설정이 꺼져 있으면 계획도 읽지 않는다", async () => {
    show(false, true);
    await waitFor(() => expect(api.settings).toHaveBeenCalled());
    expect(api.forcePushPlan).not.toHaveBeenCalled();
    expect(screen.queryByText("올린 버전을 바꿨어요")).toBeNull();
  });
});
