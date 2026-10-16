// 칸 나눔선 계산과 키 판정(D-164). 키 판정은 src/lib에만 둔다(D-056).

/** 왼쪽 칸 최소 폭(rem). 최대는 칸들 전체 폭의 50%. */
export const PANE_MIN_REM = 15;
export const PANE_MAX_RATIO = 0.5;
/** 키보드 한 번에 움직이는 폭(rem). */
export const PANE_STEP_REM = 1;

/** `min`~`max` 안으로. 최대가 최소보다 작으면(아주 좁은 창) 최소. */
export function clampPane(rem: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, rem));
}

/** 칸들 전체 폭(px)에서 최대 폭(rem). 폭을 모르면(0) 끝이 없다. */
export function paneMaxRem(containerPx: number, remPx: number): number {
  return containerPx > 0 ? (containerPx * PANE_MAX_RATIO) / remPx : Number.POSITIVE_INFINITY;
}

/** 지금 1rem의 px. 루트 글자 크기를 바꾸면(D-146) 함께 바뀐다. */
export function remPx(): number {
  return Number.parseFloat(getComputedStyle(document.documentElement).fontSize) || 16;
}

type KeyLike = { key: string; shiftKey: boolean; ctrlKey: boolean; altKey: boolean; metaKey: boolean; isComposing?: boolean };

/** ←는 -1, →는 +1, 그 밖은 0. 한글 조합 중이거나 수정키가 눌렸으면 0. */
export function paneKey(e: KeyLike): -1 | 0 | 1 {
  if (e.isComposing || e.shiftKey || e.ctrlKey || e.altKey || e.metaKey) return 0;
  if (e.key === "ArrowLeft") return -1;
  if (e.key === "ArrowRight") return 1;
  return 0;
}
