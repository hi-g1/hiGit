// 역 메뉴 항목 표(D-092, 계획 008 C11). 반복 3 항목은 이 표에 더하고 `run`에서 `runAction`으로 core
// 액션을 부른다(`{ id, input: { …, oid: info.oid } }`). 위험도·되돌리기는 action-meta와 실행기가 맡는다.
// 화면은 git을 직접 부르지 않는다(D-010).
import type { QueryClient } from "@tanstack/react-query";
import type { ReactNode } from "react";
import type { Messages } from "../../../messages";
import { Term } from "../../../ui/Term";
import type { ToastAction } from "../../../ui/toastStore";
import type { Zoom } from "../layout/simplify";
import type { StationDialog } from "./ask";
import { copyText } from "../../../lib/copy";
import type { StationInfo } from "./info";
import { runAction } from "./run";
import { MAX_RANGE } from "../range";

/** 역 밖의 화면 상태. 항목이 보일지·흐릴지 정할 때 쓴다. */
export type MenuEnv = {
  repo: string;
  currentBranch: string | null;
  /** 고친 파일 수(넣은 것 포함). */
  changedFiles: number;
  /** 이번 버전에 넣은 파일 수. */
  stagedFiles: number;
  /** 다른 도구가 시작한 작업이 멈춰 있으면 "other"(합치는 중이면 충돌 화면이 대신 열린다). */
  operation: "none" | "other";
  /** 지금 브랜치의 upstream 이름. */
  upstream: string | null;
  defaultBranch: string | null;
  zoom: Zoom;
  /** 지금 브랜치 첫 부모 선의 역 id(새 것부터, 보이는 노드 기준). */
  headLine: readonly string[];
};

export type MenuCtx = {
  repo: string;
  toast: (text: string, action?: ToastAction) => void;
  queryClient: QueryClient;
  /** 실행 뒤 결과 토스트와 되돌리기(useUndo의 notifyDone). 기록을 다시 읽고 선택은 남긴다. */
  onDone: (text: string) => void;
  /** 대화상자를 연다(태그·새 브랜치·고치기·파일 보기). */
  ask: (dialog: StationDialog) => void;
  /** 작업 내역 화면으로. */
  openActivity: () => void;
  /** 고른 버전(새 것부터)으로 기록 정리 화면을 연다. */
  openRewrite: (oids: readonly string[]) => void;
  /** 지금 브랜치 첫 부모 선(새 것부터). */
  headLine: () => readonly string[];
  m: Messages;
};

/** show: 보임, hide: 아예 없음, disabled: 흐린 글자와 툴팁의 까닭. */
export type Availability = "show" | "hide" | { disabled: (m: Messages) => string };

export type StationMenuItem = {
  id: string;
  group: "main" | "more";
  /** 용어는 `<Term focusable={false}>`(메뉴 항목 안에서 따로 포커스를 받지 않게). */
  label: (m: Messages) => ReactNode;
  /** core 액션을 부르는 항목이면 그 액션 id(action-meta.json에 있어야 한다). */
  actionId?: string;
  availability: (info: StationInfo, env: MenuEnv) => Availability;
  run: (info: StationInfo, ctx: MenuCtx) => void | Promise<void>;
};

/** 기록을 바꾸는 항목이 공통으로 흐려지는 까닭(다른 도구의 작업, 브랜치 없음). 없으면 null. */
function blockedByState(env: MenuEnv): Availability | null {
  if (env.operation === "other") return { disabled: (m) => m.whyOtherOperation };
  if (env.currentBranch === null) return { disabled: (m) => m.whyDetached };
  return null;
}

export const STATION_ITEMS: readonly StationMenuItem[] = [
  {
    // 그 버전에서 바뀐 파일과 비교(읽기만, D-105).
    id: "files",
    group: "main",
    label: (m) => m.filesAtVersion,
    availability: () => "show",
    run: (info, ctx) => ctx.ask({ kind: "files", info }),
  },
  {
    id: "copy-id",
    group: "main",
    label: (m) => m.copyVersionId,
    availability: () => "show",
    run: async (info, ctx) => {
      ctx.toast((await copyText(info.oid)) ? ctx.m.copied : ctx.m.copyFailed);
    },
  },
  {
    // 지금 위치의 버전만. 올린 버전인지는 core가 모든 원격으로 다시 본다(upstream이 없으면 흐리지 않음).
    id: "amend",
    group: "main",
    label: () => <Term id="amend" focusable={false} />,
    actionId: "amend",
    availability: (info, env) => {
      if (!info.isHead) return "hide";
      if (env.upstream !== null && !info.unpushed) return { disabled: (m) => m.whyPublished };
      return blockedByState(env) ?? "show";
    },
    run: (info, ctx) => ctx.ask({ kind: "amend", info }),
  },
  {
    // 고른 버전에서 새 브랜치를 만들고 갈아탄다. 지금 위치가 아니면 고친 파일이 섞이므로 흐린다.
    id: "new-branch",
    group: "main",
    label: (m) => m.newBranchHere,
    actionId: "branch-create",
    availability: (info, env) => {
      if (env.operation === "other") return { disabled: (m) => m.whyOtherOperation };
      if (!info.isHead && env.changedFiles > 0) return { disabled: (m) => m.whyChanged };
      return "show";
    },
    run: (info, ctx) => ctx.ask({ kind: "newBranch", info }),
  },
  {
    // 지금 위치의 조상(합쳐 들어온 곁가지 포함)만. 그 변경을 취소하는 새 버전을 만든다.
    id: "revert",
    group: "main",
    label: () => <Term id="revert" focusable={false} />,
    actionId: "revert",
    availability: (info, env) => {
      if (!info.isAncestorOfHead) return "hide";
      if (env.stagedFiles > 0) return { disabled: (m) => m.whyStaged };
      return blockedByState(env) ?? "show";
    },
    run: async (info, ctx) => {
      await runAction(ctx, { id: "revert", input: { oid: info.oid } }, { done: () => ctx.m.reverted });
    },
  },
  {
    // 다른 브랜치의 버전만(지금 위치의 조상은 이미 있다).
    id: "cherry-pick",
    group: "more",
    label: () => <Term id="cherry-pick" focusable={false} />,
    actionId: "cherry-pick",
    availability: (info, env) => {
      if (info.isAncestorOfHead) return "hide";
      if (info.isMerge) return { disabled: (m) => m.whyMergeCopy };
      if (env.stagedFiles > 0) return { disabled: (m) => m.whyStaged };
      return blockedByState(env) ?? "show";
    },
    run: async (info, ctx) => {
      await runAction(ctx, { id: "cherry-pick", input: { oid: info.oid } }, { done: () => ctx.m.picked });
    },
  },
  {
    // 그 역부터 지금 위치까지를 고른 구간으로 기록 정리 화면을 연다(D-106). 지금 위치 하나는 고치기로.
    id: "rewrite-from",
    group: "more",
    label: (m) => m.rewriteFromHere,
    actionId: "rebase-i",
    availability: (info, env) => {
      const at = env.headLine.indexOf(info.oid);
      if (!info.onHeadLine || info.isHead || info.isMerge || info.isRoot || at < 0 || at + 1 > MAX_RANGE) return "hide";
      if (env.zoom !== "all") return { disabled: (m) => m.whyZoomAll };
      if (env.operation === "other") return { disabled: (m) => m.whyOtherOperation };
      if (env.changedFiles > 0) return { disabled: (m) => m.whyChanged };
      return "show";
    },
    run: (info, ctx) => {
      // 열기 직전의 선(메뉴가 연 그 순간)에서 구간을 잡는다.
      ctx.openRewrite(ctx.headLine().slice(0, ctx.headLine().indexOf(info.oid) + 1));
    },
  },
  {
    // 태그 달기·지우기는 대화상자에서. 기록을 바꾸지 않아 다른 도구의 작업 중에만 흐린다.
    id: "tag",
    group: "more",
    label: () => <Term id="tag" focusable={false} />,
    actionId: "tag",
    availability: (_info, env) =>
      env.operation === "other" ? { disabled: (m) => m.whyOtherOperation } : "show",
    run: (info, ctx) => ctx.ask({ kind: "tag", info }),
  },
  {
    // 지금 위치의 조상이고 지금 위치가 아닌 버전. 빠지는 버전이 올라갔는지는 core가 본다.
    id: "reset",
    group: "more",
    label: () => <Term id="reset" focusable={false} />,
    actionId: "reset",
    availability: (info, env) => {
      if (!info.isAncestorOfHead || info.isHead) return "hide";
      return blockedByState(env) ?? "show";
    },
    run: async (info, ctx) => {
      await runAction(
        ctx,
        { id: "reset", input: { oid: info.oid } },
        { done: (r) => (r.kind === "reset" ? ctx.m.resetDone(r.dropped) : null) },
      );
    },
  },
];
