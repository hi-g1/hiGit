import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { GitHubStatusView } from "../../lib/api";

const status = vi.fn<() => Promise<GitHubStatusView>>();
vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: {
    githubStatus: () => status(),
    githubConnectStart: vi.fn(async () => ({ userCode: "WDJB-MJHT", expiresInSecs: 900, intervalSecs: 5 })),
    githubConnectPoll: vi.fn(),
    githubConnectCancel: vi.fn(async () => undefined),
    githubDisconnect: vi.fn(async () => undefined),
    openHostingPage: vi.fn(async () => undefined),
  },
}));

import { AppProviders, createQueryClient } from "../../App";
import { api } from "../../lib/api";
import { useToastStore } from "../../ui/toastStore";
import { useSyncStore } from "../sync/syncStore";
import { remaining } from "./ConnectPanel";
import { SettingsDialog } from "./SettingsDialog";
import { useSettingsStore } from "./settingsStore";

const notConnected: GitHubStatusView = { configured: true, connected: false, login: null, remembered: false };
const connected: GitHubStatusView = { configured: true, connected: true, login: "octocat", remembered: true };

function show() {
  act(() => useSettingsStore.getState().openSettings());
  return render(
    <AppProviders client={createQueryClient()}>
      <SettingsDialog />
    </AppProviders>,
  );
}

beforeEach(() => {
  vi.mocked(api.githubConnectPoll).mockReset();
  vi.mocked(api.githubConnectCancel).mockClear();
  vi.mocked(api.githubConnectStart).mockClear();
  vi.mocked(api.openHostingPage).mockClear();
});

afterEach(() => {
  act(() => useSettingsStore.getState().close());
  vi.useRealTimers();
});

describe("SettingsDialog", () => {
  it("구역은 일반·GitHub·정보이고 정보에 로고", async () => {
    status.mockResolvedValue(notConnected);
    show();
    const headings = screen.getAllByRole("heading", { level: 3 }).map((h) => h.textContent);
    expect(headings).toEqual(["일반", "GitHub", "정보"]);
    const info = screen.getByRole("heading", { name: "정보" }).closest("section")!;
    expect(info.querySelector('img[alt="hiGit"]')).not.toBeNull();
  });

  it("준비 안 됨이면 까닭과 꺼진 연결 단추", async () => {
    status.mockResolvedValue({ ...notConnected, configured: false });
    show();
    expect(await screen.findByText("이 앱에는 GitHub 연결이 준비되지 않았어요")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "GitHub 연결" })).toBeDisabled();
  });

  it("연결: 코드를 보이고 core가 준 간격으로 묻고, 연결되면 알린다", async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    status.mockResolvedValue(notConnected);
    useSyncStore.getState().set({ paused: true });
    vi.mocked(api.githubConnectPoll)
      .mockResolvedValueOnce({ kind: "pending", intervalSecs: 10 })
      .mockResolvedValueOnce({ kind: "connected", login: "octocat" });
    show();
    fireEvent.click(await screen.findByRole("button", { name: "GitHub 연결" }));
    expect(await screen.findByLabelText("연결 코드")).toHaveTextContent("WDJB-MJHT");
    expect(api.openHostingPage).toHaveBeenCalledWith(null, { kind: "deviceLogin" });
    expect(screen.getByText(/브라우저에서 코드를 입력해 주세요 · 1[45]:\d\d/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "GitHub 연결" })).toBeDisabled();

    await act(() => vi.advanceTimersByTimeAsync(4_900));
    expect(api.githubConnectPoll).not.toHaveBeenCalled();
    await act(() => vi.advanceTimersByTimeAsync(200));
    expect(api.githubConnectPoll).toHaveBeenCalledTimes(1);
    // 두 번째는 서버가 늘린 간격(10초) 뒤.
    await act(() => vi.advanceTimersByTimeAsync(9_000));
    expect(api.githubConnectPoll).toHaveBeenCalledTimes(1);
    status.mockResolvedValue(connected);
    await act(() => vi.advanceTimersByTimeAsync(1_100));
    expect(api.githubConnectPoll).toHaveBeenCalledTimes(2);
    await waitFor(() => expect(useToastStore.getState().toast?.text).toBe("GitHub에 연결했어요"));
    expect(useSyncStore.getState().paused).toBe(false);
    expect(await screen.findByText("@octocat")).toBeInTheDocument();
  });

  it("기다리는 중에 닫으면 묻기를 멈추고 취소를 알린다", async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    status.mockResolvedValue(notConnected);
    vi.mocked(api.githubConnectPoll).mockResolvedValue({ kind: "pending", intervalSecs: 5 });
    show();
    fireEvent.click(await screen.findByRole("button", { name: "GitHub 연결" }));
    await screen.findByLabelText("연결 코드");
    act(() => useSettingsStore.getState().close());
    expect(api.githubConnectCancel).toHaveBeenCalledTimes(1);
    await act(() => vi.advanceTimersByTimeAsync(20_000));
    expect(api.githubConnectPoll).not.toHaveBeenCalled();
  });

  it("만료·거절은 한 줄과 다시 시도", async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    status.mockResolvedValue(notConnected);
    vi.mocked(api.githubConnectPoll).mockResolvedValueOnce({ kind: "expired" });
    show();
    fireEvent.click(await screen.findByRole("button", { name: "GitHub 연결" }));
    await screen.findByLabelText("연결 코드");
    await act(() => vi.advanceTimersByTimeAsync(5_100));
    expect(await screen.findByText("코드 시간이 지났어요")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "다시 시도" }));
    await waitFor(() => expect(api.githubConnectStart).toHaveBeenCalledTimes(2));
  });

  it("시작 실패는 쉬운 말로 알린다", async () => {
    status.mockResolvedValue(notConnected);
    vi.mocked(api.githubConnectStart).mockRejectedValueOnce({ code: "githubUnreachable", detail: null });
    show();
    fireEvent.click(await screen.findByRole("button", { name: "GitHub 연결" }));
    expect(await screen.findByText("GitHub에 연결하지 못했어요. 인터넷 연결을 확인해 주세요.")).toBeInTheDocument();
  });

  it("연결됨: 로그인·저장 위치, 끊으면 권한 지우기 안내", async () => {
    status.mockResolvedValue(connected);
    show();
    expect(await screen.findByText("@octocat")).toBeInTheDocument();
    expect(screen.getByText("이 컴퓨터에 저장됨")).toBeInTheDocument();
    status.mockResolvedValue(notConnected);
    fireEvent.click(screen.getByRole("button", { name: "연결 끊기" }));
    const remove = await screen.findByRole("button", { name: "GitHub에서 권한 지우기" });
    expect(api.githubDisconnect).toHaveBeenCalled();
    fireEvent.click(remove);
    expect(api.openHostingPage).toHaveBeenCalledWith(null, { kind: "appSettings" });
  });

  it("이번 실행에만 저장된 연결", async () => {
    status.mockResolvedValue({ ...connected, remembered: false });
    show();
    expect(await screen.findByText("이번 실행에만 저장됨")).toBeInTheDocument();
  });

  it("다른 곳에서 연결을 부르면 열자마자 시작한다", async () => {
    status.mockResolvedValue(notConnected);
    act(() => useSettingsStore.getState().openSettings(true));
    render(
      <AppProviders client={createQueryClient()}>
        <SettingsDialog />
      </AppProviders>,
    );
    expect(await screen.findByLabelText("연결 코드")).toBeInTheDocument();
    expect(api.githubConnectStart).toHaveBeenCalledTimes(1);
  });

  it("남은 시간 글자", () => {
    expect(remaining(872_000)).toBe("14:32");
    expect(remaining(59_100)).toBe("1:00");
    expect(remaining(-5)).toBe("0:00");
  });

  it("상태 타입에는 토큰 칸이 없다", () => {
    type Fields = keyof GitHubStatusView;
    const exact: [Fields] extends ["configured" | "connected" | "login" | "remembered"] ? true : false = true;
    expect(exact).toBe(true);
  });
});
