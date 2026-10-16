// 웹페이지처럼 굴지 않게(D-171, D-189 ④⑤): 오른쪽 클릭 메뉴와 브라우저 단축키, 파일 끌어 놓기.
// 앱의 키 표(`RESERVED`)는 shortcuts.ts 한 곳이고 여기서는 그 표를 막기만 한다.
import { useTextMenuStore } from "../ui/textMenuStore";
import { RESERVED, isTextField, type Key } from "./shortcuts";

type Options = {
  /** 릴리스 빌드. 개발 빌드는 웹뷰 기본 메뉴와 개발자 도구 키를 열어 둔다. */
  prod: boolean;
};

/** 개발 빌드에서 막지 않는 키: 개발자 도구와 새로 고침. */
const DEV_OPEN = ["F12", "F5", "Ctrl+KeyR", "Ctrl+Shift+KeyI", "Ctrl+Shift+KeyJ"];

const keyName = (k: { code: string; ctrl: boolean; shift: boolean; alt: boolean }) =>
  `${k.ctrl ? "Ctrl+" : ""}${k.shift ? "Shift+" : ""}${k.alt ? "Alt+" : ""}${k.code}`;

/** 이 키 입력을 막아야 하는지(표에 있고, 개발 빌드의 예외가 아니면). */
export function blockedKey(e: Pick<KeyboardEvent, "code" | "ctrlKey" | "shiftKey" | "altKey">, prod: boolean): boolean {
  const name = keyName({ code: e.code, ctrl: e.ctrlKey, shift: e.shiftKey, alt: e.altKey });
  if (!prod && DEV_OPEN.includes(name)) return false;
  return RESERVED.some((k: Key) => keyName(k) === name);
}

function selectedText(target: EventTarget | null): string {
  const selection = window.getSelection();
  const text = selection?.toString() ?? "";
  if (!text || !(target instanceof Node)) return "";
  const anchor = selection?.anchorNode;
  return anchor && (anchor === target || anchor.contains(target) || target.contains(anchor)) ? text : "";
}

/** 오른쪽 클릭: 이미 처리한 곳(역·파일 줄 메뉴)은 그대로, 입력칸은 앱 메뉴, 선택한 글자는 "복사"만. */
function onContextMenu(e: MouseEvent, prod: boolean) {
  if (e.defaultPrevented) return;
  const target = e.target;
  if (isTextField(target) && (target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement)) {
    e.preventDefault();
    const fromKey = e.button !== 2 && e.clientX === 0 && e.clientY === 0;
    const box = target.getBoundingClientRect();
    useTextMenuStore.getState().open({
      kind: "field",
      field: target,
      start: target.selectionStart ?? 0,
      end: target.selectionEnd ?? 0,
      // 메뉴 키·Shift+F10은 위치가 없으니 칸 가운데에 연다.
      x: fromKey ? box.left + box.width / 2 : e.clientX,
      y: fromKey ? box.top + box.height / 2 : e.clientY,
    });
    return;
  }
  const text = selectedText(target);
  if (text) {
    e.preventDefault();
    useTextMenuStore.getState().open({ kind: "selection", text, x: e.clientX, y: e.clientY });
    return;
  }
  if (prod) e.preventDefault();
}

/** 앱 시작 때 한 번. 되돌리는 함수를 준다(시험). */
export function installWebFeel({ prod }: Options): () => void {
  const context = (e: MouseEvent) => onContextMenu(e, prod);
  const keys = (e: KeyboardEvent) => {
    if (e.isComposing || e.keyCode === 229) return;
    if (blockedKey(e, prod)) e.preventDefault();
  };
  // 파일을 창에 끌어 놓으면 웹뷰가 그 파일로 이동한다. 앱은 끌어 놓기를 쓰지 않는다.
  const drop = (e: DragEvent) => e.preventDefault();
  // contextmenu는 bubble 단계에서 본다(React가 먼저 처리해 막은 곳을 알 수 있다).
  window.addEventListener("contextmenu", context);
  window.addEventListener("keydown", keys, { capture: true });
  window.addEventListener("dragover", drop);
  window.addEventListener("drop", drop);
  return () => {
    window.removeEventListener("contextmenu", context);
    window.removeEventListener("keydown", keys, { capture: true });
    window.removeEventListener("dragover", drop);
    window.removeEventListener("drop", drop);
  };
}
