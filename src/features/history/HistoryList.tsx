import { formatRelative } from "../../lib/time";
import { useUiStore } from "../../state/uiStore";
import rowStyles from "../../ui/ListRow.module.css";
import type { HistoryRow } from "./model";

/**
 * 저장 기록 목록. 화면 모델(HistoryRow)만 받는다.
 * `now`는 상대 시각의 기준(ms). 불러온 시각을 넘겨서 다시 그려도 글자가 바뀌지 않게 한다.
 */
export function HistoryList({ rows, now }: { rows: HistoryRow[]; now: number }) {
  const locale = useUiStore((s) => s.locale);
  return (
    <ul className={rowStyles.list}>
      {rows.map((row) => (
        <li key={row.oid} className={rowStyles.row}>
          <span className={rowStyles.grow}>{row.summary}</span>
          <span className={rowStyles.muted}>
            {row.author} · {formatRelative(row.time, now, locale)}
          </span>
          <span className={`${rowStyles.mono} ${rowStyles.muted}`}>{row.shortOid}</span>
        </li>
      ))}
    </ul>
  );
}
