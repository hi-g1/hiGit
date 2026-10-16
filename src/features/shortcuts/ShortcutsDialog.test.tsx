import { act, fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it } from "vitest";
import { AppProviders, createQueryClient } from "../../App";
import { SHORTCUTS } from "../../lib/shortcuts";
import { useShortcut } from "../../lib/useShortcut";
import { ShortcutsDialog } from "./ShortcutsDialog";
import { useShortcutsStore } from "./shortcutsStore";

/** App이 하는 것과 같게 Ctrl+/로 연다. */
function Harness() {
  const setOpen = useShortcutsStore((s) => s.setOpen);
  useShortcut("shortcutHelp", () => setOpen(true));
  return <ShortcutsDialog />;
}

function show() {
  render(
    <AppProviders client={createQueryClient()}>
      <Harness />
    </AppProviders>,
  );
}

afterEach(() => act(() => useShortcutsStore.setState({ open: false })));

describe("단축키 표(D-198)", () => {
  it("Ctrl+/로 열고 표의 모든 키를 묶음별로 보인다", () => {
    show();
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    fireEvent.keyDown(window, { code: "Slash", key: "/", ctrlKey: true });
    expect(screen.getByRole("dialog", { name: "단축키" })).toBeInTheDocument();
    for (const s of Object.values(SHORTCUTS)) {
      expect(screen.getAllByText(s.label).length, s.label).toBeGreaterThan(0);
    }
    for (const group of ["어디서나", "버전 이름 칸에서", "파일 목록에서", "충돌 해결", "버전 기록"]) {
      expect(screen.getByRole("region", { name: group })).toBeInTheDocument();
    }
    expect(screen.getByText("단축키 보기")).toBeInTheDocument();
  });

  it("Esc와 표 안의 Ctrl+/로 닫는다", async () => {
    show();
    act(() => useShortcutsStore.setState({ open: true }));
    await userEvent.keyboard("{Escape}");
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    act(() => useShortcutsStore.setState({ open: true }));
    fireEvent.keyDown(screen.getByRole("button", { name: "닫기" }), { code: "Slash", key: "/", ctrlKey: true });
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });
});
