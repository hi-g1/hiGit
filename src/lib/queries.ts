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
  branches: (repo: string) => ["branches", repo] as const,
  branchName: (repo: string, name: string) => ["branchName", repo, name] as const,
  homeMap: (repo: string) => ["homeMap", repo] as const,
  mergeState: (repo: string) => ["mergeState", repo] as const,
  conflictFile: (repo: string, path: string) => ["conflictFile", repo, path] as const,
  mergeHint: (repo: string) => ["mergeHint", repo] as const,
};

/** 합치는 중인지와 충돌 파일(D-069). 앱을 켜자마자 합치는 중이면 충돌 화면을 연다. */
export function useConflictState(repo: string) {
  return useQuery<Awaited<ReturnType<typeof api.conflictState>>, ApiError>({
    queryKey: queryKeys.mergeState(repo),
    queryFn: () => api.conflictState(repo),
  });
}

/** 충돌 파일 하나의 고를 곳(D-086). */
export function useConflictFile(repo: string, path: string | null) {
  return useQuery<Awaited<ReturnType<typeof api.conflictFile>>, ApiError>({
    queryKey: queryKeys.conflictFile(repo, path ?? ""),
    queryFn: () => api.conflictFile(repo, path ?? ""),
    enabled: path !== null,
  });
}

/** 합치기 제안(D-048, D-071). */
export function useMergeHint(repo: string) {
  return useQuery<Awaited<ReturnType<typeof api.mergeHint>>, ApiError>({
    queryKey: queryKeys.mergeHint(repo),
    queryFn: () => api.mergeHint(repo),
  });
}

export function useAppStatus() {
  return useQuery<Awaited<ReturnType<typeof api.appStatus>>, ApiError>({
    queryKey: queryKeys.appStatus,
    queryFn: api.appStatus,
    staleTime: Infinity,
    // git 확인은 앱을 켤 때 한 번(안내도 "설치한 뒤 다시 실행해 주세요"). 실패한 쿼리를 새 화면이
    // 쓸 때 다시 시도하면, 시작을 기다리는 화면과 시작 화면이 번갈아 나타나며 무한히 다시 그린다.
    retryOnMount: false,
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

export function useBranches(repo: string, enabled = true) {
  return useQuery<Awaited<ReturnType<typeof api.branches>>, ApiError>({
    queryKey: queryKeys.branches(repo),
    queryFn: () => api.branches(repo),
    enabled,
  });
}

export function useBranchNameCheck(repo: string, name: string) {
  return useQuery<Awaited<ReturnType<typeof api.checkBranchName>>, ApiError>({
    queryKey: queryKeys.branchName(repo, name),
    queryFn: () => api.checkBranchName(repo, name),
    enabled: name.trim() !== "",
    staleTime: 10_000,
  });
}

export function useHomeMap(repo: string) {
  return useQuery<Awaited<ReturnType<typeof api.homeMap>>, ApiError>({
    queryKey: queryKeys.homeMap(repo),
    queryFn: () => api.homeMap(repo),
  });
}
