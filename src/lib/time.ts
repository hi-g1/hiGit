// 시각 표시 규칙(D-150). 화면의 날짜·시각은 모두 이 파일을 거친다(Intl은 여기서만, ESLint).
import type { Locale } from "../glossary";

const MINUTE = 60;
const HOUR = 3600;

function sameDay(a: Date, b: Date): boolean {
  return a.getFullYear() === b.getFullYear() && a.getMonth() === b.getMonth() && a.getDate() === b.getDate();
}

/**
 * unix 초를 화면 글자로: 1분 안·미래(시계 차이) "지금", 1시간 안 "5분 전", 같은 날 "오후 3:12", 같은 해 "10월 7일",
 * 그 전 "2025년 10월 7일". 언어를 따른다.
 */
export function formatTime(unixSeconds: number, nowMs: number, locale: Locale): string {
  const diff = Math.floor(nowMs / 1000) - unixSeconds;
  const relative = new Intl.RelativeTimeFormat(locale, { numeric: "auto" });
  if (diff < MINUTE) return relative.format(0, "second");
  if (diff < HOUR) return relative.format(-Math.floor(diff / MINUTE), "minute");
  const date = new Date(unixSeconds * 1000);
  if (sameDay(date, new Date(nowMs))) return formatClock(unixSeconds, locale, "numeric");
  return formatDate(unixSeconds, nowMs, locale);
}

/** 정확한 시각(호버). "2026년 10월 16일 오후 3:12:05". */
export function formatExact(unixSeconds: number, locale: Locale): string {
  return new Intl.DateTimeFormat(locale, { dateStyle: "long", timeStyle: "medium" }).format(unixSeconds * 1000);
}

/** 시·분만. 작업 내역 카드처럼 날짜 묶음 안에서 쓴다. */
export function formatClock(unixSeconds: number, locale: Locale, hour: "numeric" | "2-digit" = "2-digit"): string {
  return new Intl.DateTimeFormat(locale, { hour, minute: "2-digit" }).format(unixSeconds * 1000);
}

/** unix 초를 "10월 7일"로. 올해가 아니면 "2025년 10월 7일". 날짜 묶음 열쇠로도 쓴다. */
export function formatDate(unixSeconds: number, nowMs: number, locale: Locale): string {
  const date = new Date(unixSeconds * 1000);
  const sameYear = date.getFullYear() === new Date(nowMs).getFullYear();
  return new Intl.DateTimeFormat(locale, {
    year: sameYear ? undefined : "numeric",
    month: "long",
    day: "numeric",
  }).format(date);
}
