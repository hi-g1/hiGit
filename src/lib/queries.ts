// 서버 상태(TanStack Query). 화면은 이 훅으로 백엔드를 읽는다.
import { useQuery } from "@tanstack/react-query";
import { api, type ApiError, type DiffRequest } from "./api";

export const HISTORY_LIMIT = 200;

export const queryKeys = {
  appStatus: ["appStatus"] as const,
  actions: ["actions"] as const,
  recent: ["recent"] as const,
  summary: (repo: string) => ["summary", repo] as const,
  status: (repo: string) => ["status", repo] as const,
  log: (repo: string) => ["log", repo] as const,
  diff: (repo: string, path: string, untracked: boolean) => ["diff", repo, path, untracked] as const,
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

export function useDiff(repo: string, request: DiffRequest) {
  return useQuery<Awaited<ReturnType<typeof api.diff>>, ApiError>({
    queryKey: queryKeys.diff(repo, request.path, request.untracked),
    queryFn: () => api.diff(repo, request),
  });
}

export function useRecentProjects() {
  return useQuery<Awaited<ReturnType<typeof api.recentProjects>>, ApiError>({
    queryKey: queryKeys.recent,
    queryFn: api.recentProjects,
  });
}

/** 시작 화면의 프로젝트별 상태. 실패(폴더 없음, 저장소 아님)는 화면이 줄 상태로 쓴다. */
export function useProjectSummary(repo: string) {
  return useQuery<Awaited<ReturnType<typeof api.projectSummary>>, ApiError>({
    queryKey: queryKeys.summary(repo),
    queryFn: () => api.projectSummary(repo),
    staleTime: 30_000,
  });
}
