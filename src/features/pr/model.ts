// 검토 요청 줄의 글자(D-112). 상태는 색이 아니라 글자로 알린다.
import type { PrRowView } from "../../lib/api";
import type { Messages } from "../../messages";

/** 상태 글자. 열린 초안은 화면이 `<Term pr-draft>`로 보인다(`null`). */
export function stateLabel(row: PrRowView, m: Messages): string | null {
  if (row.state === "merged") return m.pr.merged;
  if (row.state === "closed") return m.pr.closed;
  return row.draft ? null : m.pr.open;
}

/** `feature → main`. 포크에서 냈으면 `소유자:feature → main`. */
export function branchLine(row: PrRowView): string {
  const head = row.headOwner ? `${row.headOwner}:${row.head}` : row.head;
  return `${head} → ${row.base}`;
}
