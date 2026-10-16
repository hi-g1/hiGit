// 화면 배율 계산(D-146). 루트 글자 크기 = 16px × 배율이고 크기 토큰은 모두 rem이라 화면 전체가 함께 커진다.
import type { DisplayHintsView, FontSizeSetting } from "./api";

/** 설정 "글자 크기"의 단계, 작은 것부터. */
export const FONT_STEPS: readonly FontSizeSetting[] = ["small", "normal", "large", "xlarge"];

/** 단계 값. 보통 + Windows 기본이면 배율 1(본문 14px). */
export const STEP_FACTOR: Record<FontSizeSetting, number> = {
  small: 13 / 14,
  normal: 1,
  large: 16 / 14,
  xlarge: 18 / 14,
};

/** 가장 작은 보조 글자(0.8125rem)가 12px 아래로 가지 않는 배율. */
export const MIN_SCALE = 12 / (16 * 0.8125);
export const MAX_SCALE = 2;

/** OS 값을 읽지 못했을 때. Windows 기본과 같다. */
export const DEFAULT_HINTS: DisplayHintsView = { bodyPx: 14, textScalePercent: 100, dark: null };

/** OS 기본 본문 px / 14 × OS 글자 배율 × 단계 값을 하한~2.0으로 자른다. */
export function computeUiScale(hints: DisplayHintsView, step: FontSizeSetting): number {
  const raw = (hints.bodyPx / 14) * (hints.textScalePercent / 100) * STEP_FACTOR[step];
  return Math.min(MAX_SCALE, Math.max(MIN_SCALE, raw));
}

/** 한 단계 크게. 끝이면 그대로. */
export function stepUp(step: FontSizeSetting): FontSizeSetting {
  return FONT_STEPS[Math.min(FONT_STEPS.length - 1, FONT_STEPS.indexOf(step) + 1)] ?? step;
}

/** 한 단계 작게. 끝이면 그대로. */
export function stepDown(step: FontSizeSetting): FontSizeSetting {
  return FONT_STEPS[Math.max(0, FONT_STEPS.indexOf(step) - 1)] ?? step;
}
