// 백엔드 이벤트 구독. 화면은 Tauri의 listen을 직접 쓰지 않고 이 파일만 쓴다.
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  OP_PROGRESS_EVENT,
  REPO_CHANGED_EVENT,
  WATCH_STATUS_EVENT,
  type OpProgress,
  type RepoChanged,
  type WatchStatus,
} from "./bindings";

export type { OpProgress, RepoChanged, WatchStatus } from "./bindings";
/** 원격 작업 id(core의 `REMOTE_ACTIONS` 그대로). 취소할 수 있고 오래 출력이 없으면 "응답 없음"이다. */
export { REMOTE_ACTIONS as REMOTE_ACTION_IDS } from "./bindings";

/** 저장소 파일이 바뀌면 불린다. 돌려준 함수로 구독을 끝낸다. */
export function onRepoChanged(handler: (event: RepoChanged) => void): Promise<UnlistenFn> {
  return listen<RepoChanged>(REPO_CHANGED_EVENT, (e) => handler(e.payload));
}

/** 감시가 켜지거나 꺼지면 불린다. */
export function onWatchStatus(handler: (event: WatchStatus) => void): Promise<UnlistenFn> {
  return listen<WatchStatus>(WATCH_STATUS_EVENT, (e) => handler(e.payload));
}

/** 오래 걸리는 작업이 도는 동안 1초마다 불린다(D-080). */
export function onOpProgress(handler: (event: OpProgress) => void): Promise<UnlistenFn> {
  return listen<OpProgress>(OP_PROGRESS_EVENT, (e) => handler(e.payload));
}
