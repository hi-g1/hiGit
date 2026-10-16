import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: {
    remoteBranchPlan: vi.fn(),
    githubStatus: vi.fn(async () => ({ configured: false, connected: false, login: null, remembered: false })),
    previewAction: vi.fn(),
    executeAction: vi.fn(),
    checkBranchName: vi.fn(),
  },
}));

import { AppProviders, createQueryClient } from "../../App";
import { api } from "../../lib/api";
import { RemoteBranchDialog } from "./RemoteBranchDialog";
import { useRemoteBranchStore } from "./remoteBranchStore";

const tip = "c".repeat(40);

afterEach(() => {
  vi.clearAllMocks();
  act(() => useRemoteBranchStore.setState({ target: null }));
});

describe("원격 브랜치 지우기 확인 창(D-130)", () => {
  it("계획이 준 끝 버전만 담아 토큰과 함께 지운다", async () => {
    vi.mocked(api.remoteBranchPlan).mockResolvedValue({
      remote: "origin",
      branch: "feature/login",
      tip,
      onlyHere: 3,
      byOthers: 1,
      blocked: null,
      isUpstreamOfCurrent: false,
    });
    vi.mocked(api.previewAction).mockResolvedValue({ token: "토큰" });
    vi.mocked(api.executeAction).mockResolvedValue({ kind: "remoteBranchDeleted", remote: "origin", branch: "feature/login" });
    act(() => useRemoteBranchStore.setState({ target: { remote: "origin", branch: "feature/login", mode: "delete" } }));
    render(
      <AppProviders client={createQueryClient()}>
        <RemoteBranchDialog repo="/repo" />
      </AppProviders>,
    );
    expect(await screen.findByText("이 브랜치에만 있는 버전 3개")).toBeInTheDocument();
    expect(screen.getByText("다른 사람의 버전 1개")).toBeInTheDocument();
    // 포커스는 안전한 "닫기"다(Enter는 창을 닫을 뿐).
    expect(screen.getByRole("button", { name: "닫기" })).toHaveFocus();
    await userEvent.click(screen.getByRole("button", { name: "지우기" }));
    const input = { id: "remote-branch-delete", input: { remote: "origin", branch: "feature/login", expectTip: tip } };
    await waitFor(() => expect(api.executeAction).toHaveBeenCalledWith("/repo", input, "토큰"));
    expect(api.previewAction).toHaveBeenCalledWith("/repo", input);
  });
});
