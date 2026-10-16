// 앱의 단축키 표(D-033, D-056). 단축키는 이 한 곳에만 둔다. 겹침은 shortcuts.test.ts가 막는다.
// 판정은 KeyboardEvent.code(물리 키)로 한다. 한글 입력 상태에서는 key가 "ㅐ"처럼 바뀐다.

export type Key = {
  code: string;
  ctrl: boolean;
  shift: boolean;
  alt: boolean;
};

export type Shortcut = {
  /** 받는 키 조합. 첫째가 화면에 보이는 키다(다시 하기는 Ctrl+Shift+Z와 Ctrl+Y). */
  keys: readonly Key[];
  /** 화면에 보이는 키 이름. 문구가 아니라 키 이름이라 messages에 두지 않는다. */
  label: string;
  /**
   * global: 창 어디서나(useShortcut). field: 그 입력칸이 직접 처리(버전 저장).
   * fileList: 파일 목록에 포커스가 있을 때만 목록이 처리.
   * conflict: 충돌 해결 화면이 열려 있을 때만(그 화면이 useShortcut을 켠다).
   * history: 버전 기록 화면이 열려 있을 때만.
   */
  scope: "global" | "field" | "fileList" | "conflict" | "history";
  /** native: 입력칸에 포커스가 있으면 처리하지 않고 브라우저 기본 동작(글자 되돌리기)에 맡긴다. */
  inTextField: "app" | "native";
};

const ctrl = (code: string, shift = false): Key => ({ code, ctrl: true, shift, alt: false });
const plain = (code: string): Key => ({ code, ctrl: false, shift: false, alt: false });

export const SHORTCUTS = {
  /** 버전 저장(이름 칸·설명 칸 안에서) */
  commit: { keys: [ctrl("Enter")], label: "Ctrl+Enter", scope: "field", inTextField: "app" },
  /** 폴더 열기(어디서나) */
  openFolder: { keys: [ctrl("KeyO")], label: "Ctrl+O", scope: "global", inTextField: "app" },
  /** 복제(어디서나, D-115) */
  clone: { keys: [ctrl("KeyO", true)], label: "Ctrl+Shift+O", scope: "global", inTextField: "app" },
  /** 방금 한 일 되돌리기(D-042). 입력칸에서는 글자 되돌리기 */
  undo: { keys: [ctrl("KeyZ")], label: "Ctrl+Z", scope: "global", inTextField: "native" },
  /** 다시 하기. Windows 사용자에게 익숙한 Ctrl+Y도 받는다(D-056) */
  redo: {
    keys: [ctrl("KeyZ", true), ctrl("KeyY")],
    label: "Ctrl+Shift+Z",
    scope: "global",
    inTextField: "native",
  },
  /** 브랜치 목록 */
  branches: { keys: [ctrl("KeyB")], label: "Ctrl+B", scope: "global", inTextField: "app" },
  /** 동기화 */
  sync: { keys: [ctrl("KeyS", true)], label: "Ctrl+Shift+S", scope: "global", inTextField: "app" },
  /** 버전 기록 화면 열기·닫기 */
  history: { keys: [ctrl("KeyH")], label: "Ctrl+H", scope: "global", inTextField: "app" },
  /** 작업 내역 화면 열기·닫기(D-085) */
  activity: { keys: [ctrl("KeyH", true)], label: "Ctrl+Shift+H", scope: "global", inTextField: "app" },
  /** 충돌 해결: 내 브랜치·합칠 브랜치·둘 다(D-086). 직접 고치는 글상자에서는 글자로 */
  chooseOurs: { keys: [plain("Digit1")], label: "1", scope: "conflict", inTextField: "native" },
  chooseTheirs: { keys: [plain("Digit2")], label: "2", scope: "conflict", inTextField: "native" },
  chooseBoth: { keys: [plain("Digit3")], label: "3", scope: "conflict", inTextField: "native" },
  /** 충돌 해결: 다음·이전 곳 */
  nextHunk: { keys: [ctrl("ArrowDown")], label: "Ctrl+↓", scope: "conflict", inTextField: "native" },
  prevHunk: { keys: [ctrl("ArrowUp")], label: "Ctrl+↑", scope: "conflict", inTextField: "native" },
  /** 버전 기록 노선도 늘리기·줄이기(D-095). 입력칸에서는 글자로. 주 조작은 화면의 −·+ 단추 */
  zoomIn: {
    keys: [{ code: "Equal", ctrl: false, shift: true, alt: false }, plain("Equal"), plain("NumpadAdd")],
    label: "+",
    scope: "history",
    inTextField: "native",
  },
  zoomOut: { keys: [plain("Minus"), plain("NumpadSubtract")], label: "-", scope: "history", inTextField: "native" },
  /** 글자 크기 한 단계 크게·작게·보통(D-138, D-152). 웹뷰 자체 확대는 꺼 두고 앱이 받는다 */
  fontLarger: {
    keys: [ctrl("Equal"), ctrl("Equal", true), ctrl("NumpadAdd")],
    label: "Ctrl+=",
    scope: "global",
    inTextField: "app",
  },
  fontSmaller: { keys: [ctrl("Minus"), ctrl("NumpadSubtract")], label: "Ctrl+-", scope: "global", inTextField: "app" },
  fontReset: { keys: [ctrl("Digit0"), ctrl("Numpad0")], label: "Ctrl+0", scope: "global", inTextField: "app" },
  /** 명령 팔레트 열기·닫기(D-170, D-199). 입력칸 안에서도 */
  palette: { keys: [ctrl("KeyK")], label: "Ctrl+K", scope: "global", inTextField: "app" },
  /** 단축키 표 열기·닫기(D-198). 입력칸 안에서도 */
  shortcutHelp: { keys: [ctrl("Slash")], label: "Ctrl+/", scope: "global", inTextField: "app" },
  /** 파일 목록의 줄 넣기·빼기(D-200) */
  toggleFile: { keys: [plain("Space")], label: "Space", scope: "fileList", inTextField: "native" },
  /** 파일 목록에서 다음·이전 파일 */
  nextFile: { keys: [plain("ArrowDown")], label: "↓", scope: "fileList", inTextField: "native" },
  prevFile: { keys: [plain("ArrowUp")], label: "↑", scope: "fileList", inTextField: "native" },
} as const satisfies Record<string, Shortcut>;

/** 아직 표에 없지만 계획된 단축키. 겹침 검사에 넣는다. */
export const PLANNED: readonly Key[] = [];

/**
 * 웹뷰(WebView2, WebKitGTK)와 OS가 쓰는 키. 앱 단축키로 쓰면 앱에 오지 않거나 동작이 섞인다.
 * Ctrl+=·−·0은 웹뷰 확대를 끄고(tauri.conf.json `zoomHotkeysEnabled: false`) 글자 크기에 쓴다(D-152).
 */
export const RESERVED: readonly Key[] = [
  ctrl("KeyF"),
  ctrl("KeyG"),
  ctrl("KeyP"),
  ctrl("KeyR"),
  ctrl("KeyU"),
  ctrl("KeyW"),
  ctrl("KeyQ"),
  ctrl("KeyI", true),
  ctrl("KeyJ", true),
  ctrl("KeyC", true),
  ctrl("KeyR", true),
  plain("F3"),
  plain("F5"),
  plain("F12"),
  { code: "ArrowLeft", ctrl: false, shift: false, alt: true },
  { code: "ArrowRight", ctrl: false, shift: false, alt: true },
];

export type ShortcutId = keyof typeof SHORTCUTS;

type KeyLike = {
  code: string;
  ctrlKey: boolean;
  shiftKey: boolean;
  altKey: boolean;
  isComposing?: boolean;
  keyCode?: number;
};

/** 이 키 입력이 단축키인지. 한글 조합 중(WebKitGTK의 keyCode 229 포함)이면 아니다. */
export function matches(id: ShortcutId, e: KeyLike): boolean {
  if (e.isComposing || e.keyCode === 229) return false;
  return SHORTCUTS[id].keys.some(
    (k) => e.code === k.code && e.ctrlKey === k.ctrl && e.shiftKey === k.shift && e.altKey === k.alt,
  );
}

/** 포커스가 글자를 쓰는 곳(입력칸, 글상자, contenteditable)인지. */
export function isTextField(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  if (target.isContentEditable) return true;
  if (target instanceof HTMLTextAreaElement) return true;
  if (target instanceof HTMLInputElement) {
    return !["checkbox", "radio", "button", "submit", "reset", "range", "color", "file"].includes(target.type);
  }
  return false;
}
