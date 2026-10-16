// 역 메뉴 항목의 core 액션 실행(D-010). 결과 토스트와 되돌리기는 `ctx.onDone`, 겹쳐 멈추면 충돌 상태를
// 다시 읽어 작업 화면이 충돌 해결 화면으로 바뀌게 하고, 실패는 오류 코드의 문구로 알린다.
import { ApiError, api, type ActionInput, type ActionResult, type ErrorCode } from "../../../lib/api";
import { invalidateLog } from "../../../lib/queries";
import type { MenuCtx } from "./items";

/** 실행에 쓰는 것만(역 메뉴 밖의 화면도 쓴다). */
export type RunCtx = Pick<MenuCtx, "repo" | "toast" | "toastError" | "queryClient" | "onDone" | "openActivity" | "m">;

export type RunOptions = {
  /** 끝났을 때 토스트 문구. null이면 토스트 없음. */
  done: (result: ActionResult) => string | null;
  /** 오류를 부른 쪽이 보여줄 때(대화상자 안). 없으면 토스트. */
  onError?: (code: ErrorCode) => void;
};

/** 작업 내역에서 되살리기를 권하는 오류(D-085, D-097). */
const POINT_TO_ACTIVITY: readonly ErrorCode[] = ["publishedHistory", "undoPushed", "undoStale"];

/** 실행 뒤 다시 읽는 조회(선택은 그대로 둔다). */
const REFRESH = ["status", "branches", "homeMap", "summary", "recovery", "mergeHint", "rewritePlan", "stashes", "mergeState"];

export async function runAction(ctx: RunCtx, input: ActionInput, options: RunOptions): Promise<ActionResult | null> {
  try {
    const result = await api.executeAction(ctx.repo, input);
    // 겹쳐 멈춘 결과는 충돌 상태를 다시 읽으면(아래) 작업 화면이 충돌 해결 화면을 연다.
    if (result.kind !== "operationConflicts" && result.kind !== "mergeConflicts") {
      const text = options.done(result);
      if (text !== null) ctx.onDone(text);
    }
    return result;
  } catch (e) {
    const code: ErrorCode = e instanceof ApiError ? e.code : "internal";
    if (options.onError) options.onError(code);
    else if (POINT_TO_ACTIVITY.includes(code)) ctx.toastError(e, { label: ctx.m.openActivity, onClick: ctx.openActivity });
    else ctx.toastError(e);
    return null;
  } finally {
    invalidateLog(ctx.queryClient, ctx.repo);
    for (const key of REFRESH) void ctx.queryClient.invalidateQueries({ queryKey: [key, ctx.repo] });
  }
}
