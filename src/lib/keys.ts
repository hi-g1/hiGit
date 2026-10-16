// Escape·Enter 판정(D-137). 화면은 키 이름을 직접 비교하지 않고 여기를 쓴다(ESLint가 막는다).
// 한글 조합 중(WebKitGTK는 keyCode 229)의 Enter·Escape는 글자를 확정하거나 조합을 끝내는 키라 명령이 아니다.

type NativeKey = { isComposing?: boolean; keyCode?: number };
/** DOM `KeyboardEvent`와 React 키 이벤트 둘 다 받는다. */
export type KeyLike = NativeKey & { key: string; shiftKey?: boolean; nativeEvent?: NativeKey };

/** 한글 조합 중인지. */
export function composing(e: NativeKey & { nativeEvent?: NativeKey }): boolean {
  const n = e.nativeEvent ?? e;
  return n.isComposing === true || n.keyCode === 229;
}

/** 닫기·취소의 Esc. 조합 중이면 아니다. */
export function isEscape(e: KeyLike): boolean {
  return e.key === "Escape" && !composing(e);
}

/** 실행의 Enter(Shift 등 조합 키와 상관없이). 조합 중이면 아니다. */
export function isEnter(e: KeyLike): boolean {
  return e.key === "Enter" && !composing(e);
}

/** 글상자에서 보내는 Enter. Shift+Enter는 줄 바꿈이라 아니다. */
export function isSubmitEnter(e: KeyLike): boolean {
  return isEnter(e) && e.shiftKey !== true;
}

/** 조합을 확정하는 Enter. 이 키로 폼이나 대화상자가 실행되지 않게 막을 때 쓴다. */
export function isComposingEnter(e: KeyLike): boolean {
  return e.key === "Enter" && composing(e);
}

/**
 * 막을 키: Enter(조합 여부와 상관없이)와 조합 중인 모든 키. Enter로 아무것도 하지 않는 칸(되돌릴 수 없는 일의 이름
 * 칸 등)이 쓴다.
 */
export function isBlockedEnter(e: KeyLike): boolean {
  return e.key === "Enter" || composing(e);
}
