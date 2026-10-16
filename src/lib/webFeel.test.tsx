import { act, fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import { AppProviders, createQueryClient } from "../App";
import { useTextMenuStore } from "../ui/textMenuStore";
import { RESERVED } from "./shortcuts";
import { blockedKey, installWebFeel } from "./webFeel";

const key = (code: string, mods: { ctrl?: boolean; shift?: boolean; alt?: boolean } = {}) => ({
  code,
  ctrlKey: mods.ctrl ?? false,
  shiftKey: mods.shift ?? false,
  altKey: mods.alt ?? false,
});

let uninstall: (() => void) | null = null;
afterEach(() => {
  uninstall?.();
  uninstall = null;
  act(() => useTextMenuStore.setState({ target: null, pasteBlocked: false }));
});

describe("브라우저 단축키 막기(D-189 ⑤)", () => {
  it("릴리스 빌드는 표의 키를 모두 막는다", () => {
    for (const k of RESERVED) {
      expect(blockedKey(key(k.code, k), true), k.code).toBe(true);
    }
    expect(blockedKey(key("F5"), true)).toBe(true);
    expect(blockedKey(key("KeyR", { ctrl: true }), true)).toBe(true);
    expect(blockedKey(key("ArrowLeft", { alt: true }), true)).toBe(true);
  });

  it("개발 빌드는 개발자 도구와 새로 고침을 열어 두고 나머지는 막는다", () => {
    expect(blockedKey(key("F12"), false)).toBe(false);
    expect(blockedKey(key("KeyI", { ctrl: true, shift: true }), false)).toBe(false);
    expect(blockedKey(key("F5"), false)).toBe(false);
    expect(blockedKey(key("KeyP", { ctrl: true }), false)).toBe(true);
  });

  it("앱 단축키는 막지 않는다", () => {
    expect(blockedKey(key("KeyZ", { ctrl: true }), true)).toBe(false);
    expect(blockedKey(key("KeyB", { ctrl: true }), true)).toBe(false);
    expect(blockedKey(key("KeyC", { ctrl: true }), true)).toBe(false);
  });

  it("설치하면 keydown 캡처에서 막는다", () => {
    uninstall = installWebFeel({ prod: true });
    const blocked = new KeyboardEvent("keydown", { code: "F5", cancelable: true, bubbles: true });
    window.dispatchEvent(blocked);
    expect(blocked.defaultPrevented).toBe(true);
    const allowed = new KeyboardEvent("keydown", { code: "KeyZ", ctrlKey: true, cancelable: true, bubbles: true });
    window.dispatchEvent(allowed);
    expect(allowed.defaultPrevented).toBe(false);
  });
});

describe("오른쪽 클릭(D-189 ④)", () => {
  function setup(prod = true) {
    uninstall = installWebFeel({ prod });
    return render(
      <AppProviders client={createQueryClient()}>
        <button type="button">단추</button>
        <input aria-label="칸" defaultValue="안녕하세요" />
        <div onContextMenu={(e) => e.preventDefault()}>직접 처리</div>
      </AppProviders>,
    );
  }

  it("버튼 위는 웹뷰 메뉴를 막고 앱 메뉴도 열지 않는다", () => {
    setup();
    const event = fireEvent.contextMenu(screen.getByRole("button", { name: "단추" }));
    expect(event).toBe(false);
    expect(useTextMenuStore.getState().target).toBeNull();
  });

  it("개발 빌드에서는 버튼 위 웹뷰 메뉴를 막지 않는다", () => {
    setup(false);
    expect(fireEvent.contextMenu(screen.getByRole("button", { name: "단추" }))).toBe(true);
  });

  it("입력칸 위는 앱 메뉴(잘라내기·복사·붙여넣기·모두 선택)", async () => {
    setup();
    fireEvent.contextMenu(screen.getByRole("textbox", { name: "칸" }), { clientX: 10, clientY: 20 });
    expect(useTextMenuStore.getState().target?.kind).toBe("field");
    for (const name of ["잘라내기", "복사", "붙여넣기", "모두 선택"]) {
      expect(await screen.findByRole("menuitem", { name: new RegExp(name) })).toBeInTheDocument();
    }
  });

  it("이미 직접 처리한 곳은 그대로 둔다", () => {
    setup();
    fireEvent.contextMenu(screen.getByText("직접 처리"));
    expect(useTextMenuStore.getState().target).toBeNull();
  });

  it("모두 선택은 웹뷰 편집 명령으로 하고 닫히면 입력칸으로 돌아간다", async () => {
    setup();
    const exec = vi.fn(() => true);
    Object.defineProperty(document, "execCommand", { value: exec, configurable: true });
    const field = screen.getByRole("textbox", { name: "칸" });
    fireEvent.contextMenu(field, { clientX: 10, clientY: 20 });
    await userEvent.click(await screen.findByRole("menuitem", { name: /모두 선택/ }));
    expect(exec).toHaveBeenCalledWith("selectAll");
    expect(field).toHaveFocus();
  });

  it("클립보드 읽기를 못 하면 붙여넣기가 꺼지고 까닭을 말한다", async () => {
    setup();
    Object.defineProperty(navigator, "clipboard", { value: undefined, configurable: true });
    fireEvent.contextMenu(screen.getByRole("textbox", { name: "칸" }), { clientX: 10, clientY: 20 });
    const paste = await screen.findByRole("menuitem", { name: /붙여넣기/ });
    expect(paste).toHaveAttribute("aria-disabled", "true");
  });
});
