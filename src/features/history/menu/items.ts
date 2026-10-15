// 역 메뉴 항목 표(D-092). 반복 3은 이 표에 항목을 더하고 `run`에서 core 액션을 부르면 된다
// (`api.executeAction({ id, input: { …, oid: info.oid } })`). 위험도·되돌리기는 action-meta와 실행기가
// 맡는다. 화면은 git을 직접 부르지 않는다(D-010).
import type { QueryClient } from "@tanstack/react-query";
import type { Messages } from "../../../messages";
import { copyText } from "./copy";
import type { StationInfo } from "./info";

/** 역 밖의 화면 상태. 반복 3 항목(예: 고친 파일이 있으면 되돌리기 끄기)이 쓴다. */
export type MenuEnv = {
  repo: string;
  currentBranch: string | null;
  changedFiles: number;
  merging: boolean;
};

export type MenuCtx = {
  repo: string;
  toast: (text: string) => void;
  queryClient: QueryClient;
  /** 실행 뒤 결과 토스트와 되돌리기(useUndo의 notifyDone). 기록을 다시 읽고 선택은 남긴다. */
  onDone: (text: string) => void;
  m: Messages;
};

/** show: 보임, hide: 아예 없음, disabled: 흐린 글자와 툴팁의 까닭. */
export type Availability = "show" | "hide" | { disabled: (m: Messages) => string };

export type StationMenuItem = {
  id: string;
  group: "main" | "more";
  label: (m: Messages) => string;
  /** core 액션을 부르는 항목이면 그 액션 id(action-meta.json에 있어야 한다). */
  actionId?: string;
  availability: (info: StationInfo, env: MenuEnv) => Availability;
  run: (info: StationInfo, ctx: MenuCtx) => void | Promise<void>;
};

export const STATION_ITEMS: readonly StationMenuItem[] = [
  {
    id: "copy-id",
    group: "main",
    label: (m) => m.copyVersionId,
    availability: () => "show",
    run: async (info, ctx) => {
      ctx.toast((await copyText(info.oid)) ? ctx.m.copied : ctx.m.copyFailed);
    },
  },
];
