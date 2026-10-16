import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const setSettings = vi.fn();
vi.mock("./api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("./api")>()),
  api: { setSettings: (p: unknown) => setSettings(p) },
}));

import { createQueryClient } from "../App";
import { DEFAULT_SETTINGS } from "./display";
import { queryKeys } from "./queries";
import { changeSettings, currentSettings, SAVE_DELAY_MS } from "./settingsSave";

beforeEach(() => {
  vi.useFakeTimers();
  setSettings.mockReset();
  setSettings.mockImplementation(async (p: object) => ({ ...DEFAULT_SETTINGS, ...p }));
});

afterEach(() => {
  vi.useRealTimers();
});

describe("설정 바꾸기", () => {
  it("화면 값은 바로, 저장은 모아서 한 번", async () => {
    const client = createQueryClient();
    changeSettings(client, { fontSize: "large" });
    changeSettings(client, { fontSize: "xlarge" });
    changeSettings(client, { diffWrap: false });
    expect(currentSettings(client)).toMatchObject({ fontSize: "xlarge", diffWrap: false });
    expect(setSettings).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(SAVE_DELAY_MS);
    expect(setSettings).toHaveBeenCalledTimes(1);
    expect(setSettings).toHaveBeenCalledWith({ fontSize: "xlarge", diffWrap: false });
    expect(client.getQueryData(queryKeys.settings)).toMatchObject({ fontSize: "xlarge", diffWrap: false });
  });

  it("안내 닫기·다시 보기는 기다리지 않고 따로 보내고 화면 값도 맞춘다(D-205)", async () => {
    const client = createQueryClient();
    changeSettings(client, { tipSeen: "firstSync" });
    changeSettings(client, { tipSeen: "firstUndo" });
    expect(setSettings).toHaveBeenNthCalledWith(1, { tipSeen: "firstSync" });
    expect(setSettings).toHaveBeenNthCalledWith(2, { tipSeen: "firstUndo" });
    expect(currentSettings(client).tipsSeen).toMatchObject({ firstSync: true, firstUndo: true, firstBranch: false });
    expect(currentSettings(client)).not.toHaveProperty("tipSeen");
    changeSettings(client, { resetTips: true });
    expect(setSettings).toHaveBeenLastCalledWith({ resetTips: true });
    expect(Object.values(currentSettings(client).tipsSeen).some(Boolean)).toBe(false);
  });

  it("저장하지 못해도 화면 값은 남는다", async () => {
    setSettings.mockRejectedValue(new Error("x"));
    const client = createQueryClient();
    changeSettings(client, { fontSize: "small" });
    await vi.advanceTimersByTimeAsync(SAVE_DELAY_MS);
    expect(currentSettings(client).fontSize).toBe("small");
  });
});
