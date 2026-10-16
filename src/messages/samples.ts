// 문구 시험이 함께 쓰는 예시 인자(copy-rules, parity). 함수형 문구를 불러 모든 문장을 문자열로 만든다.

/** 상태 종류를 받는 문구는 종류마다 불러 모든 문장을 검사한다. 나머지는 ("예시", 3). */
export const SAMPLES: Record<string, unknown[][]> = {
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
  routeRowLabel: [
    [
      {
        summary: "로그인 화면",
        author: "지원",
        date: "10월 7일",
        branches: ["기능"],
        tags: ["v1"],
        head: true,
        merge: true,
        unpushed: true,
        position: 3,
      },
    ],
  ],
};

/** 메시지 객체 안의 모든 문자열. 함수형 문구는 예시 인자로 불러 문자열로 만든다. */
export function messageStrings(value: unknown, path = ""): [string, string][] {
  if (typeof value === "string") return [[path, value]];
  if (typeof value === "function") {
    const key = path.split(".").at(-1) ?? "";
    const calls = SAMPLES[key] ?? [["예시", 3]];
    return calls.map((args, i): [string, string] => [`${path}#${i}`, String(value(...args))]);
  }
  if (value && typeof value === "object") {
    return Object.entries(value).flatMap(([k, v]) => messageStrings(v, path ? `${path}.${k}` : k));
  }
  return [];
}
