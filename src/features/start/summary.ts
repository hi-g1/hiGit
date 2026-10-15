// 시작 화면의 상태 문장·짧은 상태 고르기(순수 함수). 문장은 messages가 만든다(D-031, D-029).
import type { ProjectSummaryView } from "../../lib/api";

/** 상태 문장의 앞부분 */
export type StatusCase =
  | { kind: "conflict"; files: number }
  | { kind: "changed"; files: number; branch: string }
  | { kind: "clean"; branch: string }
  | { kind: "detached" };

/** 상태 문장의 뒷부분 */
export type LastVersion = { kind: "last"; time: number; summary: string } | { kind: "none" };

/** 목록 줄의 짧은 상태(하나만). 충돌만 빨강으로 보인다. */
export type ShortCase =
  | { kind: "conflict"; files: number }
  | { kind: "changed"; files: number }
  | { kind: "ahead"; versions: number }
  | { kind: "behind"; versions: number }
  | { kind: "notUploaded" }
  | { kind: "upToDate" };

export function pickStatus(s: ProjectSummaryView): StatusCase {
  if (s.conflictedFiles > 0) return { kind: "conflict", files: s.conflictedFiles };
  if (s.branch.name === null) return { kind: "detached" };
  if (s.changedFiles > 0) return { kind: "changed", files: s.changedFiles, branch: s.branch.name };
  return { kind: "clean", branch: s.branch.name };
}

export function pickLastVersion(s: ProjectSummaryView): LastVersion {
  return s.lastCommit
    ? { kind: "last", time: s.lastCommit.authorTime, summary: s.lastCommit.summary }
    : { kind: "none" };
}

export function pickShort(s: ProjectSummaryView): ShortCase {
  if (s.conflictedFiles > 0) return { kind: "conflict", files: s.conflictedFiles };
  if (s.changedFiles > 0) return { kind: "changed", files: s.changedFiles };
  if (s.branch.upstream) {
    if (s.branch.ahead > 0) return { kind: "ahead", versions: s.branch.ahead };
    if (s.branch.behind > 0) return { kind: "behind", versions: s.branch.behind };
    return { kind: "upToDate" };
  }
  return s.branch.hasCommits ? { kind: "notUploaded" } : { kind: "upToDate" };
}

/** main·master가 아닐 때만 목록 줄에 브랜치 이름을 붙인다(계획 003 Q12). */
export function branchPrefix(s: ProjectSummaryView): string | null {
  const name = s.branch.name;
  return name && name !== "main" && name !== "master" ? name : null;
}
