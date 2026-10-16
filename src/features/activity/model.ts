// 작업 내역 화면의 계산(D-085). DOM 없이 시험한다.
import type { TermId } from "../../glossary/types";
import type { ActionInput, RecoveryItem } from "../../lib/api";

export type DayGroup = { day: string; items: RecoveryItem[] };

/** 같은 날(지역 시각)끼리 묶는다. 조회가 이미 최신 먼저로 준다. */
export function groupByDay(items: RecoveryItem[], dayOf: (at: number) => string): DayGroup[] {
  const groups: DayGroup[] = [];
  for (const item of items) {
    const day = dayOf(item.at);
    const last = groups[groups.length - 1];
    if (last && last.day === day) last.items.push(item);
    else groups.push({ day, items: [item] });
  }
  return groups;
}

/** 줄의 단추가 부를 액션 입력. 되살릴 수 없으면 `null`. 값은 조회가 준 것만 돌려보낸다. */
export function restoreInput(item: RecoveryItem): ActionInput | null {
  // 앱 작업 줄은 그 작업 전 버전으로(D-204). core가 지금 브랜치에서 HEAD를 옮긴 줄에만 값을 준다.
  if (item.source === "journal") return item.beforeOid ? { id: "restore", input: { oid: item.beforeOid } } : null;
  if (item.restore === "version" && item.oid) return { id: "restore", input: { oid: item.oid } };
  if (item.restore === "changes") {
    if (item.source === "stash" && item.oid) return { id: "take-out", input: { source: { kind: "stash", oid: item.oid } } };
    if (item.refname) return { id: "take-out", input: { source: { kind: "ref", refname: item.refname } } };
  }
  return null;
}

/** 앱 작업 줄 아래의 Ctrl+Z 글자(D-204). 되돌린 줄과 앱 작업이 아닌 줄은 없다. */
export function undoHintKey(item: RecoveryItem): "next" | "earlier" | "never" | "revertCommit" | null {
  if (item.source !== "journal") return null;
  return item.undo === "done" || item.undo === "none" ? null : item.undo;
}

/** 백업 이름이 곧 액션 id인 것(실행기가 HEAD를 남길 때 쓰는 이름). */
const ACTION_LABELS: readonly TermId[] = ["reset", "amend", "rebase-i", "cherry-pick", "revert", "discard", "lock"];
/** 백업 이름의 앞부분과 그 용어. 뒤는 브랜치 이름(긴 것부터 맞춘다). */
const PREFIX_LABELS: readonly [string, TermId][] = [
  ["conflict-files-", "conflict"],
  ["branch-delete-", "branch-delete"],
  ["restore-", "reflog"],
  ["discard-", "discard"],
  ["switch-carry-", "switch"],
  ["parked-", "park"],
  ["undo-", "reflog"],
];

/**
 * 백업 ref 이름(마지막 조각, `/`는 `%2F`로 적힘)을 용어와 이름으로. 모르는 이름이면 `null`(원문 그대로
 * 보인다).
 */
export function backupLabel(label: string): { term: TermId; name: string | null } | null {
  const decoded = label.replace(/%2F/g, "/").replace(/%25/g, "%");
  const exact = ACTION_LABELS.find((id) => id === decoded);
  if (exact) return { term: exact, name: null };
  for (const [prefix, term] of PREFIX_LABELS) {
    if (decoded.startsWith(prefix) && decoded.length > prefix.length) return { term, name: decoded.slice(prefix.length) };
  }
  return null;
}
