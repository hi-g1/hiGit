import { act, fireEvent, render, screen } from "@testing-library/react";
import type { CSSProperties } from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const setSettings = vi.fn();
vi.mock("./api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("./api")>()),
  api: { setSettings: (p: object) => setSettings(p), settings: vi.fn(async () => ({ ...DEFAULTS })) },
}));

import { AppProviders, createQueryClient } from "../App";
import { Splitter } from "../ui/Splitter";
import { DEFAULT_SETTINGS as DEFAULTS } from "./display";
import { queryKeys } from "./queries";
import { SAVE_DELAY_MS } from "./settingsSave";
import { usePaneWidth, type PaneKey } from "./usePaneWidth";

function Pane({ which }: { which: PaneKey }) {
  const pane = usePaneWidth(which, 21.25);
  return (
    <div data-testid={which} style={{ "--pane-width": `${pane.rem}rem` } as CSSProperties}>
      <Splitter label={which} {...pane.splitter} />
    </div>
  );
}

beforeEach(() => {
  vi.useFakeTimers({ shouldAdvanceTime: true });
  setSettings.mockReset();
  setSettings.mockImplementation(async (p: object) => ({ ...DEFAULTS, ...p }));
});
afterEach(() => {
  vi.useRealTimers();
});

describe("칸 폭 저장(D-164)", () => {
  it("저장된 폭으로 열고, 바꾸면 그 화면 칸에만 rem으로 저장한다", async () => {
    const client = createQueryClient();
    client.setQueryData(queryKeys.settings, { ...DEFAULTS, workspacePane: 25, conflictPane: null });
    render(
      <AppProviders client={client}>
        <Pane which="workspacePane" />
        <Pane which="conflictPane" />
      </AppProviders>,
    );
    expect(screen.getByTestId("workspacePane").style.getPropertyValue("--pane-width")).toBe("25rem");
    expect(screen.getByTestId("conflictPane").style.getPropertyValue("--pane-width")).toBe("21.25rem");
    fireEvent.keyDown(screen.getByRole("separator", { name: "workspacePane" }), { key: "ArrowRight" });
    expect(screen.getByTestId("workspacePane").style.getPropertyValue("--pane-width")).toBe("26rem");
    await act(() => vi.advanceTimersByTimeAsync(SAVE_DELAY_MS));
    expect(setSettings).toHaveBeenCalledWith({ workspacePane: 26 });
    expect(screen.getByTestId("conflictPane").style.getPropertyValue("--pane-width")).toBe("21.25rem");
  });
});
