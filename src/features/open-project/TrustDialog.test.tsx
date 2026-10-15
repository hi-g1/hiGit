import { act, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: { trustProject: vi.fn(), openProject: vi.fn() },
}));

import { AppProviders, createQueryClient } from "../../App";
import { api } from "../../lib/api";
import { useRepoStore } from "../../state/repoStore";
import { TrustDialog } from "./TrustDialog";
import { useTrustStore } from "./trustStore";

afterEach(() => {
  vi.clearAllMocks();
  act(() => {
    useTrustStore.setState({ pending: null });
    useRepoStore.setState({ repo: null });
  });
});

function show() {
  render(
    <AppProviders client={createQueryClient()}>
      <TrustDialog />
    </AppProviders>,
  );
  act(() => useTrustStore.getState().ask({ folder: "/받은/프로젝트", keys: ["core.fsmonitor", "hooks/post-checkout"] }));
}

describe("신뢰 확인", () => {
  it("실행되는 설정 이름을 보여주고 끄고 열기를 고르면 연다", async () => {
    vi.mocked(api.trustProject).mockResolvedValue({ kind: "opened", repo: { path: "/받은/프로젝트", name: "프로젝트" } });
    show();
    expect(screen.getByText("core.fsmonitor")).toBeInTheDocument();
    expect(screen.getByText("hooks/post-checkout")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "끄고 열기" }));
    expect(api.trustProject).toHaveBeenCalledWith("/받은/프로젝트", "disable");
    await vi.waitFor(() => expect(useRepoStore.getState().repo?.path).toBe("/받은/프로젝트"));
    expect(useTrustStore.getState().pending).toBeNull();
  });

  it("믿고 열기", async () => {
    vi.mocked(api.trustProject).mockResolvedValue({ kind: "opened", repo: { path: "/받은/프로젝트", name: "프로젝트" } });
    show();
    await userEvent.click(screen.getByRole("button", { name: "믿고 열기" }));
    expect(api.trustProject).toHaveBeenCalledWith("/받은/프로젝트", "trust");
  });

  it("Esc로 닫으면 열지 않는다", async () => {
    show();
    await userEvent.keyboard("{Escape}");
    expect(api.trustProject).not.toHaveBeenCalled();
    expect(useTrustStore.getState().pending).toBeNull();
  });
});
