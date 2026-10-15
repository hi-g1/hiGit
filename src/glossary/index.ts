import { en } from "./en";
import { ko } from "./ko";
import type { Glossary, Locale, TermEntry, TermId } from "./types";

export type { Locale, TermEntry, TermId } from "./types";
export { TERM_IDS } from "./types";

const glossaries: Record<Locale, Glossary> = { ko, en };

export function getTerm(id: TermId, locale: Locale = "ko"): TermEntry {
  return glossaries[locale][id];
}

export function isTermId(value: string): value is TermId {
  return Object.hasOwn(ko, value);
}
