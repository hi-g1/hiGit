import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const setSettings = vi.fn();
vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: {
    appStatus: vi.fn(async () => ({ gitVersion: "2.45.0", gitSecurity: "ok", gitPackageVersion: null, gitPath: "C:\\Program Files\\Git\\cmd\\git.exe" })),
    openLogFolder: vi.fn(async () => null),
    settings: vi.fn(async () => ({ ...DEFAULTS })),
    setSettings: (p: object) => setSettings(p),
  },
}));

import { AppProviders, createQueryClient } from "../../App";
import { api } from "../../lib/api";
import { DEFAULT_SETTINGS as DEFAULTS } from "../../lib/display";
import { SAVE_DELAY_MS } from "../../lib/settingsSave";
import { AdvancedSection } from "./AdvancedSection";

function show() {
  render(
    <AppProviders client={createQueryClient()}>
      <AdvancedSection />
    </AppProviders>,
  );
}

beforeEach(() => {
  vi.useFakeTimers({ shouldAdvanceTime: true });
  setSettings.mockReset();
  setSettings.mockImplementation(async (p: object) => ({ ...DEFAULTS, ...p }));
});
afterEach(() => vi.useRealTimers());

describe("설정 고급 구역(D-160)", () => {
  it("Git 위치를 읽기 전용으로 보이고 로그 폴더를 연다", async () => {
    show();
    expect(screen.getByRole("heading", { name: "고급" })).toBeInTheDocument();
    expect(await screen.findByText("C:\\Program Files\\Git\\cmd\\git.exe")).toBeInTheDocument();
    expect(screen.queryByRole("textbox")).toBeNull();
    await userEvent.click(screen.getByRole("button", { name: "로그 폴더 열기" }));
    expect(api.openLogFolder).toHaveBeenCalledTimes(1);
  });

  it("올린 버전도 바꾸기는 기본 꺼짐이고 켜면 확인 없이 저장한다(D-127)", async () => {
    show();
    const box = await screen.findByRole("checkbox", { name: "올린 버전도 바꾸기" });
    expect(box).not.toBeChecked();
    expect(box).toHaveAccessibleDescription("덮어쓰며 올리기도 켜져요. 원격 저장소의 기본 브랜치는 바꾸지 않아요.");
    await userEvent.click(box);
    expect(box).toBeChecked();
    expect(screen.queryByRole("dialog")).toBeNull();
    await act(async () => vi.advanceTimersByTime(SAVE_DELAY_MS));
    await waitFor(() => expect(setSettings).toHaveBeenCalledWith({ allowPublishedRewrite: true }));
  });
});
