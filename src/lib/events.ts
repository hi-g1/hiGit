// 백엔드 이벤트 구독. 화면은 Tauri의 listen을 직접 쓰지 않고 이 파일만 쓴다.
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  REPO_CHANGED_EVENT,
  WATCH_STATUS_EVENT,
  type RepoChanged,
  type WatchStatus,
} from "./bindings";

export type { RepoChanged, WatchStatus } from "./bindings";

/** 저장소 파일이 바뀌면 불린다. 돌려준 함수로 구독을 끝낸다. */
export function onRepoChanged(handler: (event: RepoChanged) => void): Promise<UnlistenFn> {
  return listen<RepoChanged>(REPO_CHANGED_EVENT, (e) => handler(e.payload));
}

/** 감시가 켜지거나 꺼지면 불린다. */
export function onWatchStatus(handler: (event: WatchStatus) => void): Promise<UnlistenFn> {
  return listen<WatchStatus>(WATCH_STATUS_EVENT, (e) => handler(e.payload));
}
