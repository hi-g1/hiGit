// 화면 언어 고르기(D-139, D-149). OS 언어는 웹뷰의 navigator.languages로 본다(WebView2는 Windows 표시 언어,
// WebKitGTK는 로케일 환경). navigator.language(s)는 이 파일에서만 읽는다(ESLint).
import type { Locale } from "../glossary";
import type { LanguageSetting } from "./api";

/** 설정과 OS 언어 목록으로 화면 언어를 정한다. 시스템 따라가기는 첫 언어가 한국어면 한국어, 그 밖은 English. */
export function resolveLocale(setting: LanguageSetting, languages: readonly string[]): Locale {
  if (setting === "ko" || setting === "en") return setting;
  return languages[0]?.toLowerCase().startsWith("ko") ? "ko" : "en";
}

/** 웹뷰가 알려 주는 OS 언어 목록. */
export function systemLanguages(): readonly string[] {
  if (typeof navigator === "undefined") return [];
  return navigator.languages?.length ? navigator.languages : navigator.language ? [navigator.language] : [];
}
