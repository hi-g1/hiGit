import { createContext, useContext, type ReactNode } from "react";
import type { ErrorCode } from "../lib/api";

/**
 * 오류 코드별 해결 단추(예: 이름·이메일이 없으면 "이름과 이메일 설정"). 앱 맨 위(`AppProviders`)가 채운다.
 * `ui`가 기능 코드를 모르게 context로 받는다.
 */
export type ErrorActions = Partial<Record<ErrorCode, ReactNode>>;

export const ErrorActionsContext = createContext<ErrorActions>({});

export function useErrorAction(code: ErrorCode): ReactNode {
  return useContext(ErrorActionsContext)[code];
}
