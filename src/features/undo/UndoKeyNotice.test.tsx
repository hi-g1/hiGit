import { act, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: { appStatus: vi.fn() },
}));

import { AppProviders, createQueryClient } from "../../App";
import { api } from "../../lib/api";
import { UndoKeyNotice, useUndoKeyNoticeStore } from "./UndoKeyNotice";

const status = (undoKey: "saved" | "temporary") => ({
  gitVersion: "2.45.0",
  gitSecurity: "ok" as const,
  gitPackageVersion: null,
  gitPath: "/usr/bin/git",
  undoKey,
  undoKeyDetail: undoKey === "temporary" ? "~/.local/share/dev.higit.desktop" : null,
});

function show() {
  return render(
    <AppProviders client={createQueryClient()}>
      <UndoKeyNotice />
      <UndoKeyNotice />
    </AppProviders>,
  );
}

afterEach(() => {
  vi.clearAllMocks();
  act(() => useUndoKeyNoticeStore.setState({ dismissed: false }));
});

describe("되돌리기 키 알림(D-210)", () => {
  it("임시 키면 알리고 자세히 보기에 폴더, 닫으면 이번 실행 동안 두 곳 모두 숨긴다", async () => {
    vi.mocked(api.appStatus).mockResolvedValue(status("temporary"));
    show();
    const lines = await screen.findAllByRole("status");
    expect(lines).toHaveLength(2);
    expect(lines[0]).toHaveTextContent("되돌리기 기록을 이 컴퓨터에 저장하지 못해요.");
    expect(screen.getAllByText("~/.local/share/dev.higit.desktop")).toHaveLength(2);
    await userEvent.click(screen.getAllByRole("button", { name: "닫기" })[0]!);
    expect(screen.queryAllByRole("status")).toHaveLength(0);
  });

  it("저장된 키면 아무것도 없다", async () => {
    vi.mocked(api.appStatus).mockResolvedValue(status("saved"));
    const { container } = show();
    await vi.waitFor(() => expect(api.appStatus).toHaveBeenCalled());
    expect(container.textContent).toBe("");
  });
});
