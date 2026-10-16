// @vitest-environment node
// 오류 문구가 시키는 행동은 화면에 실제로 있는 버튼 이름이어야 한다(docs/research/ux.md 1장 "높음").
// 예: 화면에는 "동기화" 버튼뿐인데 "가져오기부터 해 주세요"라고 하지 않는다.
import { describe, expect, it } from "vitest";
import { getTerm, type Locale, type TermId } from "../glossary";
import { en } from "./en";
import { ko } from "./ko";

const t = (id: TermId, locale: Locale) => getTerm(id, locale).friendly;

/** 오류 코드 → 그 문구가 가리키는 실제 버튼 이름. */
const ACTIONS: Record<string, (locale: Locale) => string> = {
  pushRejected: (l) => t("sync", l),
  notPushed: (l) => t("sync", l),
  stagedChanges: (l) => t("commit", l),
  hasChanges: (l) => t("stash", l),
  wouldOverwrite: (l) => t("commit", l),
  // 합치기 제안의 "main 가져와 합치기"(브랜치 목록의 "main에 합치기"가 앞으로 감을 수 없을 때).
  notFastForward: (l) => (l === "ko" ? ko.bringMerge("").trim() : t("merge", l)),
};

/** 화면에 버튼이 없는 행동. 오류 문구에 쓰지 않는다(동기화·자동 원격 확인이 대신한다, D-045, D-059). */
const NO_BUTTON: TermId[] = ["pull", "fetch"];

describe("오류 문구 속 행동 이름", () => {
  it.each(Object.entries(ACTIONS))("%s는 실제 버튼 이름을 쓴다", (code, label) => {
    expect(ko.errors[code as keyof typeof ko.errors]).toContain(label("ko"));
    expect(en.errors[code as keyof typeof en.errors]).toContain(label("en"));
  });

  it("버튼이 없는 행동(가져오기·원격 확인)을 시키지 않는다", () => {
    for (const [locale, errors] of [
      ["ko", ko.errors],
      ["en", en.errors],
    ] as const) {
      for (const [code, text] of Object.entries(errors)) {
        for (const id of NO_BUTTON) {
          expect(text.includes(t(id, locale)), `${locale} ${code}: "${text}"`).toBe(false);
        }
      }
    }
  });
});
