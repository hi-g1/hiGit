import { afterEach, describe, expect, it, vi } from "vitest";

const settings = vi.fn();
const displayHints = vi.fn();
vi.mock("./api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("./api")>()),
  api: { settings: () => settings(), displayHints: () => displayHints() },
}));

import { createQueryClient } from "../App";
import { ApiError } from "./api";
import { useUiStore } from "../state/uiStore";
import { DEFAULT_SETTINGS } from "./display";
import { preloadDisplay } from "./preload";
import { queryKeys } from "./queries";

afterEach(() => {
  vi.useRealTimers();
});

describe("시작 때 화면 설정 미리 읽기", () => {
  it("설정과 OS 값을 캐시에 넣고 루트 글자 크기를 정한다", async () => {
    settings.mockResolvedValue({ ...DEFAULT_SETTINGS, fontSize: "large" });
    displayHints.mockResolvedValue({ bodyPx: 15, textScalePercent: 100, dark: null });
    const client = createQueryClient();
    await preloadDisplay(client);
    expect(client.getQueryData(queryKeys.settings)).toMatchObject({ fontSize: "large" });
    expect(useUiStore.getState().uiScale).toBeCloseTo((15 / 14) * (16 / 14));
  });

  it("Git이 없어 설정을 못 읽어도 OS 배율은 적용한다", async () => {
    settings.mockRejectedValue(new ApiError({ code: "gitMissing", detail: null }));
    displayHints.mockResolvedValue({ bodyPx: 14, textScalePercent: 150, dark: null });
    await preloadDisplay(createQueryClient());
    expect(useUiStore.getState().uiScale).toBeCloseTo(1.5);
  });

  it("늦으면 기다리지 않고 기본값", async () => {
    vi.useFakeTimers();
    settings.mockReturnValue(new Promise(() => {}));
    displayHints.mockReturnValue(new Promise(() => {}));
    const done = preloadDisplay(createQueryClient(), 800);
    await vi.advanceTimersByTimeAsync(800);
    await done;
    expect(useUiStore.getState().uiScale).toBe(1);
  });
});
