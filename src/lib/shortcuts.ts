// 앱의 단축키 표(D-033). 단축키는 이 한 곳에만 둔다. 겹침은 shortcuts.test.ts가 막는다.
// 판정은 KeyboardEvent.code(물리 키)로 한다. 한글 입력 상태에서는 key가 "ㅐ"처럼 바뀐다.

export type Shortcut = {
  code: string;
  ctrl: boolean;
  shift: boolean;
  alt: boolean;
  /** 화면에 보이는 키 이름. 문구가 아니라 키 이름이라 messages에 두지 않는다. */
  label: string;
};

export const SHORTCUTS = {
  /** 버전 저장(저장 상자 안에서) */
  commit: { code: "Enter", ctrl: true, shift: false, alt: false, label: "Ctrl+Enter" },
  /** 폴더 열기(어디서나) */
  openFolder: { code: "KeyO", ctrl: true, shift: false, alt: false, label: "Ctrl+O" },
  // 반복 4: clone: { code: "KeyO", ctrl: true, shift: true, alt: false, label: "Ctrl+Shift+O" },
  // 계획: commandPalette: { code: "KeyK", ctrl: true, shift: false, alt: false, label: "Ctrl+K" },
} as const satisfies Record<string, Shortcut>;

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
  const s = SHORTCUTS[id];
  return e.code === s.code && e.ctrlKey === s.ctrl && e.shiftKey === s.shift && e.altKey === s.alt;
}
