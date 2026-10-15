// 저장 기록 화면 모델. 목록(반복 0)과 나중의 노선도 보기가 같이 쓴다
// (docs/architecture.md "버전 기록 그리기 계층"). 배치 계산·그리기는 여기 두지 않는다.
import type { LogPage } from "../../lib/api";

export type HistoryRow = {
  oid: string;
  shortOid: string;
  summary: string;
  author: string;
  /** unix 초 */
  time: number;
  parents: string[];
  isMerge: boolean;
};

export function toHistoryRows(page: LogPage): HistoryRow[] {
  return page.commits.map((c) => ({
    oid: c.oid,
    shortOid: c.shortOid,
    summary: c.summary,
    author: c.authorName,
    time: c.authorTime,
    parents: c.parents,
    isMerge: c.parents.length > 1,
  }));
}
