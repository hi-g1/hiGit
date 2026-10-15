import type { Locale } from "../glossary";

const UNITS: [Intl.RelativeTimeFormatUnit, number][] = [
  ["year", 365 * 24 * 3600],
  ["month", 30 * 24 * 3600],
  ["week", 7 * 24 * 3600],
  ["day", 24 * 3600],
  ["hour", 3600],
  ["minute", 60],
];

/** unix 초를 "3시간 전" 같은 상대 시각으로. 1분 안쪽은 "지금". */
export function formatRelative(unixSeconds: number, nowMs: number, locale: Locale): string {
  const diff = unixSeconds - Math.floor(nowMs / 1000);
  const format = new Intl.RelativeTimeFormat(locale, { numeric: "auto" });
  for (const [unit, seconds] of UNITS) {
    if (Math.abs(diff) >= seconds) {
      return format.format(Math.round(diff / seconds), unit);
    }
  }
  return format.format(0, "second");
}
