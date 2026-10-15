// 테스트용 버전 기록 한 쪽. 필요한 칸만 덮어쓴다.
import type { LogPage } from "../lib/api";

export function logPage(over: Partial<LogPage> = {}): LogPage {
  return {
    commits: [],
    hasMore: false,
    labels: [],
    tips: [],
    tipsCut: 0,
    head: null,
    currentBranch: null,
    defaultBranch: null,
    defaultTip: null,
    unpushed: [],
    ...over,
  };
}
