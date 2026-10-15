// @vitest-environment node
// 화면 문구 규칙(D-024, D-027): 괄호 보충 금지, 옛 용어 금지, 해요체.
import { describe, expect, it } from "vitest";
import { en as glossaryEn } from "../glossary/en";
import { ko as glossaryKo } from "../glossary/ko";
import { en } from "./en";
import { ko } from "./ko";

/** D-024 이전의 쉬운 이름. 화면 문구에 다시 쓰지 않는다. */
const OLD_TERMS = [
  "갈래",
  "저장하기",
  "저장에 넣",
  "저장에서 빼",
  "저장 기록",
  "새 소식",
  "최신으로 받기",
  "안전망",
  "잠시 치워두기",
  "겹친 부분",
  "바뀐 내용",
];

/** 상태 종류를 받는 문구는 종류마다 불러 모든 문장을 검사한다. 나머지는 ("예시", 3). */
const SAMPLES: Record<string, unknown[][]> = {
  statusSentence: [
    [{ kind: "conflict", files: 2 }],
    [{ kind: "changed", files: 3, branch: "main" }],
    [{ kind: "clean", branch: "main" }],
    [{ kind: "detached" }],
  ],
  lastVersionSentence: [
    [{ kind: "last", time: 0, summary: "비밀번호 확인 추가" }, "10월 7일"],
    [{ kind: "none" }, ""],
  ],
  shortStatus: [
    [{ kind: "conflict", files: 2 }],
    [{ kind: "changed", files: 3 }],
    [{ kind: "ahead", versions: 1 }],
    [{ kind: "behind", versions: 5 }],
    [{ kind: "notUploaded" }],
    [{ kind: "upToDate" }],
  ],
  diffSummary: [[{ added: 1, removed: 2 }]],
};

/** 메시지 객체 안의 모든 문자열. 함수형 문구는 예시 인자로 불러 문자열로 만든다. */
function strings(value: unknown, path = ""): [string, string][] {
  if (typeof value === "string") return [[path, value]];
  if (typeof value === "function") {
    const key = path.split(".").at(-1) ?? "";
    const calls = SAMPLES[key] ?? [["예시", 3]];
    return calls.map((args, i): [string, string] => [`${path}#${i}`, String(value(...args))]);
  }
  if (value && typeof value === "object") {
    return Object.entries(value).flatMap(([k, v]) => strings(v, path ? `${path}.${k}` : k));
  }
  return [];
}

function glossaryStrings(glossary: typeof glossaryKo): [string, string][] {
  return Object.entries(glossary).flatMap(([id, t]) => [
    [`${id}.friendly`, t.friendly],
    [`${id}.description`, t.description],
  ]);
}

const korean = [...strings(ko), ...glossaryStrings(glossaryKo)];
const all = [...korean, ...strings(en), ...glossaryStrings(glossaryEn)];

describe("문구 규칙", () => {
  it("괄호로 덧붙인 설명이 없다", () => {
    for (const [path, text] of all) {
      expect(text, path).not.toMatch(/[()（）]/);
    }
  });

  it("옛 용어를 쓰지 않는다", () => {
    for (const [path, text] of korean) {
      for (const old of OLD_TERMS) {
        expect(text.includes(old), `${path}: "${text}"에 "${old}"`).toBe(false);
      }
    }
  });

  it("관리 시작은 버전 관리 시작으로만 쓴다", () => {
    for (const [path, text] of korean) {
      expect(text.replaceAll("버전 관리 시작", "").includes("관리 시작"), path).toBe(false);
    }
  });

  it("한국어 문구는 해요체다", () => {
    for (const [path, text] of strings(ko)) {
      expect(text, path).not.toMatch(/니다[.!?]?$/);
    }
  });
});
