// 서버 상태(TanStack Query). 화면은 이 훅으로 백엔드를 읽는다.
import { useQuery } from "@tanstack/react-query";
import { api, type ApiError } from "./api";

export const HISTORY_LIMIT = 200;

export const queryKeys = {
  appStatus: ["appStatus"] as const,
  actions: ["actions"] as const,
  status: (repo: string) => ["status", repo] as const,
  log: (repo: string) => ["log", repo] as const,
};

export function useAppStatus() {
  return useQuery<Awaited<ReturnType<typeof api.appStatus>>, ApiError>({
    queryKey: queryKeys.appStatus,
    queryFn: api.appStatus,
    staleTime: Infinity,
  });
}

export function useActions() {
  return useQuery<Awaited<ReturnType<typeof api.listActions>>, ApiError>({
    queryKey: queryKeys.actions,
    queryFn: api.listActions,
    staleTime: Infinity,
  });
}

export function useStatus(repo: string) {
  return useQuery<Awaited<ReturnType<typeof api.status>>, ApiError>({
    queryKey: queryKeys.status(repo),
    queryFn: () => api.status(repo),
  });
}

export function useLog(repo: string) {
  return useQuery<Awaited<ReturnType<typeof api.log>>, ApiError>({
    queryKey: queryKeys.log(repo),
    queryFn: () => api.log(repo, { skip: 0, limit: HISTORY_LIMIT }),
  });
}
