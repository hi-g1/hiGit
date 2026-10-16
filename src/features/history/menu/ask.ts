// 역 메뉴 항목이 여는 대화상자(계획 008 C11). 항목은 `ctx.ask(…)`로 열고, 버전 기록 화면이 상태로 쥐고
// `StationDialogs`가 그린다.
import type { StationInfo } from "./info";

export type StationDialog =
  | { kind: "tag"; info: StationInfo }
  | { kind: "newBranch"; info: StationInfo }
  | { kind: "amend"; info: StationInfo }
  | { kind: "files"; info: StationInfo };
