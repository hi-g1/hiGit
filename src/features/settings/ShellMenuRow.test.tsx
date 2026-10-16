import { act, render, renderHook, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactNode } from "react";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: {
    settings: vi.fn(),
    shellMenuStatus: vi.fn(),
    shellMenuSet: vi.fn(),
    shellMenuSync: vi.fn(),
  },
}));

import { AppProviders, createQueryClient } from "../../App";
import { ApiError, api } from "../../lib/api";
import { DEFAULT_SETTINGS } from "../../lib/display";
import { queryKeys } from "../../lib/queries";
import { useDisplaySync } from "../../lib/useDisplaySync";
import { ShellMenuRow } from "./ShellMenuRow";
import { useShellMenuSync } from "./useShellMenuSync";

afterEach(() => vi.clearAllMocks());

function show(client = createQueryClient()) {
  client.setQueryData(queryKeys.settings, { ...DEFAULT_SETTINGS, language: "ko" });
  render(
    <AppProviders client={client}>
      <ShellMenuRow />
    </AppProviders>,
  );
  return client;
}

describe("폴더 오른쪽 클릭 메뉴 설정(D-214)", () => {
  it("실제 등록 상태와 종류별 설명을 보이고 켜고 끈다", async () => {
    vi.mocked(api.shellMenuStatus).mockResolvedValue({ kind: "nautilusScripts", installed: true });
    vi.mocked(api.shellMenuSet).mockResolvedValue({ kind: "nautilusScripts", installed: false });
    show();
    const box = await screen.findByRole("checkbox", { name: "폴더 오른쪽 클릭 메뉴" });
    expect(box).toBeChecked();
    expect(api.shellMenuStatus).toHaveBeenCalledWith("Git으로 관리");
    expect(screen.getByText("파일 앱 오른쪽 클릭 메뉴의 스크립트에 넣어요.")).toBeInTheDocument();
    await userEvent.click(box);
    expect(api.shellMenuSet).toHaveBeenCalledWith(false, "Git으로 관리");
    await waitFor(() => expect(box).not.toBeChecked());
  });

  it("이 컴퓨터에서 못 쓰면 행이 없다", async () => {
    vi.mocked(api.shellMenuStatus).mockResolvedValue({ kind: "unsupported", installed: false });
    show();
    await waitFor(() => expect(api.shellMenuStatus).toHaveBeenCalled());
    expect(screen.queryByRole("checkbox", { name: "폴더 오른쪽 클릭 메뉴" })).not.toBeInTheDocument();
  });

  it("켤 때 실패하면 오류를 보인다", async () => {
    vi.mocked(api.shellMenuStatus).mockResolvedValue({ kind: "explorer", installed: false });
    vi.mocked(api.shellMenuSet).mockRejectedValue(new ApiError({ code: "shellMenuFailed", detail: "PermissionDenied" }));
    show();
    expect(await screen.findByText("탐색기에서 폴더를 오른쪽 클릭해 바로 열어요.")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("checkbox", { name: "폴더 오른쪽 클릭 메뉴" }));
    await waitFor(() => expect(api.shellMenuSet).toHaveBeenCalledWith(true, "Git으로 관리"));
    expect(await screen.findByRole("alert")).toBeInTheDocument();
  });

  it("켤 때와 언어를 바꿨을 때 이름을 넘겨 맞춘다", async () => {
    vi.mocked(api.shellMenuSync).mockResolvedValue({ kind: "explorer", installed: true });
    const client = createQueryClient();
    client.setQueryData(queryKeys.settings, { ...DEFAULT_SETTINGS, language: "ko" });
    const wrapper = ({ children }: { children: ReactNode }) => <AppProviders client={client}>{children}</AppProviders>;
    renderHook(
      () => {
        useDisplaySync();
        useShellMenuSync(true);
      },
      { wrapper },
    );
    await waitFor(() => expect(api.shellMenuSync).toHaveBeenCalledWith("Git으로 관리"));
    act(() => client.setQueryData(queryKeys.settings, { ...DEFAULT_SETTINGS, language: "en" }));
    await waitFor(() => expect(api.shellMenuSync).toHaveBeenCalledWith("Manage with Git"));
    expect(client.getQueryData(queryKeys.shellMenu("Manage with Git"))).toEqual({ kind: "explorer", installed: true });
  });

  it("Git이 없으면(앱 상태를 못 얻으면) 맞추지 않는다", () => {
    renderHook(() => useShellMenuSync(false), {
      wrapper: ({ children }: { children: ReactNode }) => <AppProviders client={createQueryClient()}>{children}</AppProviders>,
    });
    expect(api.shellMenuSync).not.toHaveBeenCalled();
  });
});
