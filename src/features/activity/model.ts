// 작업 내역 화면의 계산(D-085). DOM 없이 시험한다.
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
  if (item.restore === "version" && item.oid) return { id: "restore", input: { oid: item.oid } };
  if (item.restore === "changes") {
    if (item.source === "stash" && item.oid) return { id: "take-out", input: { source: { kind: "stash", oid: item.oid } } };
    if (item.refname) return { id: "take-out", input: { source: { kind: "ref", refname: item.refname } } };
  }
  return null;
}
