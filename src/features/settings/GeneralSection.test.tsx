import { act, fireEvent, render, renderHook, screen, waitFor, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const setSettings = vi.fn();
vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: {
    settings: vi.fn(async () => ({ ...DEFAULTS })),
    setSettings: (p: object) => setSettings(p),
  },
}));

import type { ReactNode } from "react";
import { AppProviders, createQueryClient } from "../../App";
import { DEFAULT_SETTINGS as DEFAULTS } from "../../lib/display";
import { queryKeys } from "../../lib/queries";
import { currentSettings, SAVE_DELAY_MS } from "../../lib/settingsSave";
import { GeneralSection } from "./GeneralSection";
import { useDisplaySync } from "../../lib/useDisplaySync";
import { useFontShortcuts } from "./useFontShortcuts";

function DisplaySync() {
  useDisplaySync();
  return null;
}

beforeEach(() => {
  vi.useFakeTimers({ shouldAdvanceTime: true });
  setSettings.mockReset();
  setSettings.mockImplementation(async (p: object) => ({ ...DEFAULTS, ...p }));
});

afterEach(() => {
  vi.useRealTimers();
});

const press = (code: string, mods: KeyboardEventInit = {}) =>
  act(() => {
    window.dispatchEvent(new KeyboardEvent("keydown", { code, ctrlKey: true, cancelable: true, ...mods }));
  });

describe("설정 일반 구역", () => {
  it("글자 크기를 고르면 바로 바뀌고 저장된다", async () => {
    const client = createQueryClient();
    render(
      <AppProviders client={client}>
        <GeneralSection />
      </AppProviders>,
    );
    expect(screen.getByRole("heading", { name: "일반" })).toBeInTheDocument();
    const group = await screen.findByRole("radiogroup", { name: "글자 크기" });
    expect(group).toBeInTheDocument();
    expect(screen.getByRole("radio", { name: "보통" })).toHaveAttribute("aria-checked", "true");
    fireEvent.click(screen.getByRole("radio", { name: "아주 크게" }));
    await waitFor(() => expect(screen.getByRole("radio", { name: "아주 크게" })).toHaveAttribute("aria-checked", "true"));
    await act(() => vi.advanceTimersByTimeAsync(SAVE_DELAY_MS));
    expect(setSettings).toHaveBeenCalledWith({ fontSize: "xlarge" });
  });

  it("언어를 고르면 바로 영어가 되고 저장된다", async () => {
    const client = createQueryClient();
    client.setQueryData(queryKeys.settings, { ...DEFAULTS, language: "ko" });
    render(
      <AppProviders client={client}>
        <DisplaySync />
        <GeneralSection />
      </AppProviders>,
    );
    fireEvent.click(screen.getByRole("radio", { name: "English" }));
    expect(await screen.findByRole("heading", { name: "General" })).toBeInTheDocument();
    expect(document.documentElement.lang).toBe("en");
    await act(() => vi.advanceTimersByTimeAsync(SAVE_DELAY_MS));
    expect(setSettings).toHaveBeenCalledWith({ language: "en" });
  });

  it("화면 모드를 고르면 바로 바뀌고 저장된다", async () => {
    const client = createQueryClient();
    client.setQueryData(queryKeys.settings, { ...DEFAULTS });
    render(
      <AppProviders client={client}>
        <DisplaySync />
        <GeneralSection />
      </AppProviders>,
    );
    const group = screen.getByRole("radiogroup", { name: "화면 모드" });
    fireEvent.click(within(group).getByRole("radio", { name: "다크" }));
    await waitFor(() => expect(document.documentElement.dataset.theme).toBe("dark"));
    await act(() => vi.advanceTimersByTimeAsync(SAVE_DELAY_MS));
    expect(setSettings).toHaveBeenCalledWith({ theme: "dark" });
    delete document.documentElement.dataset.theme;
  });

  it("용어 설명 보기를 끄면 저장된다", async () => {
    const client = createQueryClient();
    client.setQueryData(queryKeys.settings, { ...DEFAULTS });
    render(
      <AppProviders client={client}>
        <GeneralSection />
      </AppProviders>,
    );
    const box = screen.getByRole("checkbox", { name: "용어 설명 보기" });
    expect(box).toBeChecked();
    fireEvent.click(box);
    await waitFor(() => expect(screen.getByRole("checkbox", { name: "용어 설명 보기" })).not.toBeChecked());
    await act(() => vi.advanceTimersByTimeAsync(SAVE_DELAY_MS));
    expect(setSettings).toHaveBeenCalledWith({ termHelp: false });
  });

  it("Ctrl+=·Ctrl+−·Ctrl+0이 글자 크기를 한 칸씩 바꾸고 끝에서는 저장하지 않는다", async () => {
    const client = createQueryClient();
    client.setQueryData(queryKeys.settings, { ...DEFAULTS });
    const wrapper = ({ children }: { children: ReactNode }) => <AppProviders client={client}>{children}</AppProviders>;
    renderHook(() => useFontShortcuts(), { wrapper });
    press("Equal");
    expect(currentSettings(client).fontSize).toBe("large");
    press("Equal", { shiftKey: true });
    expect(currentSettings(client).fontSize).toBe("xlarge");
    press("NumpadAdd");
    expect(currentSettings(client).fontSize).toBe("xlarge");
    await act(() => vi.advanceTimersByTimeAsync(SAVE_DELAY_MS));
    expect(setSettings).toHaveBeenCalledTimes(1);
    expect(setSettings).toHaveBeenLastCalledWith({ fontSize: "xlarge" });
    press("Minus");
    expect(currentSettings(client).fontSize).toBe("large");
    press("Digit0");
    expect(currentSettings(client).fontSize).toBe("normal");
    // 조합 중이면 무시
    act(() => {
      window.dispatchEvent(new KeyboardEvent("keydown", { code: "Equal", ctrlKey: true, isComposing: true }));
    });
    expect(currentSettings(client).fontSize).toBe("normal");
    // 입력칸 안에서도 동작한다.
    const input = document.createElement("input");
    document.body.appendChild(input);
    act(() => {
      input.dispatchEvent(new KeyboardEvent("keydown", { code: "Minus", ctrlKey: true, bubbles: true, cancelable: true }));
    });
    expect(currentSettings(client).fontSize).toBe("small");
    input.remove();
  });
});
