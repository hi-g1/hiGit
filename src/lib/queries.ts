// 서버 상태(TanStack Query). 화면은 이 훅으로 백엔드를 읽는다.
import { type InfiniteData, type QueryClient, useInfiniteQuery, useQuery } from "@tanstack/react-query";
import {
  api,
  type ApiError,
  type DiffRequest,
  type LogPage,
  type LogScope,
  type PrListView,
  type PrStateFilterView,
} from "./api";
import { limiter } from "./limit";

export const HISTORY_LIMIT = 200;

export const queryKeys = {
  appStatus: ["appStatus"] as const,
  actions: ["actions"] as const,
  recent: ["recent"] as const,
  summary: (repo: string) => ["summary", repo] as const,
  status: (repo: string) => ["status", repo] as const,
  log: (repo: string) => ["log", repo] as const,
  diff: (repo: string, path: string, untracked: boolean, commit: string, fullContext = false) =>
    ["diff", repo, path, untracked, commit, fullContext] as const,
  commitDetail: (repo: string, oid: string) => ["commitDetail", repo, oid] as const,
  rewritePlan: (repo: string, oids: readonly string[]) => ["rewritePlan", repo, oids.join(",")] as const,
  forcePushPlan: (repo: string) => ["forcePushPlan", repo] as const,
  remoteBranchPlan: (repo: string, remote: string, branch: string) => ["remoteBranchPlan", repo, remote, branch] as const,
  branches: (repo: string) => ["branches", repo] as const,
  branchName: (repo: string, name: string) => ["branchName", repo, name] as const,
  homeMap: (repo: string) => ["homeMap", repo] as const,
  mergeState: (repo: string) => ["mergeState", repo] as const,
  conflictFile: (repo: string, path: string) => ["conflictFile", repo, path] as const,
  mergeHint: (repo: string) => ["mergeHint", repo] as const,
  recovery: (repo: string) => ["recovery", repo] as const,
  stashes: (repo: string) => ["stashes", repo] as const,
  pendingWork: (repo: string) => ["pendingWork", repo] as const,
  routePreview: (repo: string, limit: number) => ["routePreview", repo, limit] as const,
  /** GitHub 연결에 기대는 것은 모두 "github"로 시작한다(연결·끊기 뒤 한 번에 무효화). */
  github: ["github"] as const,
  githubStatus: ["github", "status"] as const,
  hostingContext: (repo: string) => ["github", "context", repo] as const,
  prList: (repo: string, state: string, branch: string | null) => ["github", "prList", repo, state, branch] as const,
  prDetail: (repo: string, number: number) => ["github", "prDetail", repo, number] as const,
  prDraft: (repo: string) => ["github", "prDraft", repo] as const,
  cloneDefaults: ["cloneDefaults"] as const,
  settings: ["settings"] as const,
  displayHints: ["displayHints"] as const,
  identity: ["identity"] as const,
};

/** 앱 설정(D-127). */
export function useAppSettings() {
  return useQuery<Awaited<ReturnType<typeof api.settings>>, ApiError>({
    queryKey: queryKeys.settings,
    queryFn: api.settings,
    staleTime: Infinity,
  });
}

/** 전역 이름·이메일(D-125). 저장하면 다시 읽는다. */
export function useIdentity() {
  return useQuery<Awaited<ReturnType<typeof api.identity>>, ApiError>({
    queryKey: queryKeys.identity,
    queryFn: () => api.identity(),
    staleTime: 60_000,
  });
}

/** OS 글자 크기·배율·다크(D-147). 창으로 돌아오면 5초에 한 번까지 다시 읽어 OS 설정을 따른다. */
export function useDisplayHints() {
  return useQuery<Awaited<ReturnType<typeof api.displayHints>>, ApiError>({
    queryKey: queryKeys.displayHints,
    queryFn: api.displayHints,
    staleTime: 5000,
    refetchOnWindowFocus: true,
    retry: false,
  });
}

/** 복제 대화상자의 부모 폴더 기본값(홈 폴더). */
export function useCloneDefaults() {
  return useQuery<Awaited<ReturnType<typeof api.cloneDefaults>>, ApiError>({
    queryKey: queryKeys.cloneDefaults,
    queryFn: api.cloneDefaults,
    staleTime: Infinity,
  });
}

/** 열린 저장소의 호스팅 정보(네트워크 없음). */
export function useHostingContext(repo: string, enabled = true) {
  return useQuery<Awaited<ReturnType<typeof api.hostingContext>>, ApiError>({
    queryKey: queryKeys.hostingContext(repo),
    queryFn: () => api.hostingContext(repo),
    enabled,
  });
}

/** 검토 요청 목록(쪽마다 "더 보기", D-118). 화면이 열려 있을 때만, 60초 동안 새로 읽지 않는다. */
export function usePrPages(repo: string, state: PrStateFilterView, enabled: boolean) {
  return useInfiniteQuery<
    PrListView,
    ApiError,
    InfiniteData<PrListView, number>,
    ReturnType<typeof queryKeys.prList>,
    number
  >({
    queryKey: queryKeys.prList(repo, state, null),
    queryFn: ({ pageParam }) => api.prList(repo, { state, branch: null, page: pageParam }),
    initialPageParam: 1,
    getNextPageParam: (last, pages) => (last.hasMore ? pages.length + 1 : undefined),
    staleTime: 60_000,
    enabled,
  });
}

/** 한 브랜치의 검토 요청(브랜치 목록, D-071). 목록이 열릴 때만, 5분 동안 새로 읽지 않는다. */
export function usePrForBranch(repo: string, branch: string | null, enabled: boolean) {
  return useQuery<PrListView, ApiError>({
    queryKey: queryKeys.prList(repo, "all", branch),
    queryFn: () => api.prList(repo, { state: "all", branch, page: 1 }),
    staleTime: 5 * 60_000,
    enabled: enabled && branch !== null,
  });
}

/** 새 검토 요청 대화상자의 기본값(네트워크 없음). 열 때마다 새로 읽는다. */
export function usePrDraft(repo: string) {
  return useQuery<Awaited<ReturnType<typeof api.prDraft>>, ApiError>({
    queryKey: queryKeys.prDraft(repo),
    queryFn: () => api.prDraft(repo),
    staleTime: 0,
  });
}

export function usePrDetail(repo: string, number: number | null) {
  return useQuery<Awaited<ReturnType<typeof api.prDetail>>, ApiError>({
    queryKey: queryKeys.prDetail(repo, number ?? 0),
    queryFn: () => api.prDetail(repo, number ?? 0),
    enabled: number !== null,
    staleTime: 60_000,
  });
}

/** 설정의 GitHub 구역 상태(키체인을 건드리지 않는다). */
export function useGitHubStatus() {
  return useQuery<Awaited<ReturnType<typeof api.githubStatus>>, ApiError>({
    queryKey: queryKeys.githubStatus,
    queryFn: api.githubStatus,
  });
}

/** 시작 화면 노선도는 동시에 3개까지 부른다(Windows는 git 시작이 느리다, D-093). */
export const previewQueue = limiter(3);

/** 시작 화면 노선도(D-093). 상태 요약이 성공한 프로젝트만(`enabled`). */
export function useRoutePreview(repo: string, limit: number, enabled: boolean) {
  return useQuery<Awaited<ReturnType<typeof api.routePreview>>, ApiError>({
    queryKey: queryKeys.routePreview(repo, limit),
    queryFn: () => previewQueue(() => api.routePreview(repo, limit)),
    enabled,
    staleTime: 30_000,
  });
}

/** 끝나지 않은 작업(D-082). 오래된 잠금, 남은 맡긴 항목, 다른 도구의 작업. */
export function usePendingWork(repo: string) {
  return useQuery<Awaited<ReturnType<typeof api.pendingWork>>, ApiError>({
    queryKey: queryKeys.pendingWork(repo),
    queryFn: () => api.pendingWork(repo),
    // 잠금 파일은 60초가 지나야 버려진 것으로 본다. 그사이 파일 변경 알림이 없을 수 있어 다시 묻는다.
    refetchInterval: 60_000,
  });
}

/** 임시 보관 목록(D-076). */
export function useStashes(repo: string) {
  return useQuery<Awaited<ReturnType<typeof api.stashes>>, ApiError>({
    queryKey: queryKeys.stashes(repo),
    queryFn: () => api.stashes(repo),
  });
}

/** 작업 내역(D-085). */
export function useRecovery(repo: string) {
  return useQuery<Awaited<ReturnType<typeof api.recovery>>, ApiError>({
    queryKey: queryKeys.recovery(repo),
    queryFn: () => api.recovery(repo),
  });
}

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

/** 받아 둘 버전 수 상한. 넘으면 더 받지 않는다. */
export const HISTORY_MAX = 20_000;

type LogPageParam = { skip: number; tips: string[] | null };

/**
 * 버전 기록 쪽들(D-088). 다음 쪽은 첫 쪽이 정한 끝점(`tips`)과 받은 수(`skip`)로 이어 받아 그사이
 * ref가 움직여도 겹치거나 빠지지 않는다.
 */
export function useLogPages(repo: string, scope: LogScope) {
  return useInfiniteQuery<LogPage, ApiError, InfiniteData<LogPage, LogPageParam>, readonly unknown[], LogPageParam>({
    queryKey: [...queryKeys.log(repo), scope],
    initialPageParam: { skip: 0, tips: null },
    queryFn: ({ pageParam }) =>
      api.log(repo, { skip: pageParam.skip, limit: HISTORY_LIMIT, scope, tips: pageParam.tips }),
    getNextPageParam: (last, pages) => {
      const received = pages.reduce((n, p) => n + p.commits.length, 0);
      if (!last.hasMore || received >= HISTORY_MAX) return undefined;
      return { skip: received, tips: pages[0]!.tips };
    },
  });
}

/**
 * 버전 기록을 다시 읽게 한다. 받은 쪽을 첫 쪽 하나로 자른 뒤 무효화해, 2만 개를 받아 둔 상태에서도
 * 첫 쪽 한 번만 다시 받는다(useInfiniteQuery는 무효화하면 모든 쪽을 다시 받는다).
 */
export function invalidateLog(client: QueryClient, repo: string): void {
  client.setQueriesData<InfiniteData<LogPage, LogPageParam>>({ queryKey: queryKeys.log(repo) }, (data) =>
    data && data.pages.length > 1 ? { pages: data.pages.slice(0, 1), pageParams: data.pageParams.slice(0, 1) } : data,
  );
  void client.invalidateQueries({ queryKey: queryKeys.log(repo) });
}

export function useDiff(repo: string, request: DiffRequest, options: { enabled?: boolean } = {}) {
  const commit = request.commit ?? "";
  return useQuery<Awaited<ReturnType<typeof api.diff>>, ApiError>({
    queryKey: queryKeys.diff(repo, request.path, request.untracked, commit, request.fullContext ?? false),
    queryFn: () => api.diff(repo, request),
    enabled: options.enabled ?? true,
    // 버전 안의 비교는 바뀌지 않는다.
    staleTime: commit ? Infinity : 0,
  });
}

/** 기록 정리 카드 지도의 처음 모습(D-106). `.git`이 바뀌면 다시 읽는다(막힌 까닭이 바뀔 수 있다). */
export function useRewritePlan(repo: string, oids: readonly string[]) {
  return useQuery<Awaited<ReturnType<typeof api.rewritePlan>>, ApiError>({
    queryKey: queryKeys.rewritePlan(repo, oids),
    queryFn: () => api.rewritePlan(repo, [...oids]),
    enabled: oids.length > 0,
  });
}

/** 덮어쓰며 올리기 확인 창의 값(D-128). 설정이 켜졌을 때만 읽는다(꺼져 있으면 진입점이 없다). */
export function useForcePushPlan(repo: string, enabled: boolean) {
  return useQuery<Awaited<ReturnType<typeof api.forcePushPlan>>, ApiError>({
    queryKey: queryKeys.forcePushPlan(repo),
    queryFn: () => api.forcePushPlan(repo),
    enabled,
  });
}

/** 원격 브랜치 지우기·이름 바꾸기 확인 창의 값(D-130). 창이 열렸을 때만 읽는다. */
export function useRemoteBranchPlan(repo: string, remote: string, branch: string, enabled: boolean) {
  return useQuery<Awaited<ReturnType<typeof api.remoteBranchPlan>>, ApiError>({
    queryKey: queryKeys.remoteBranchPlan(repo, remote, branch),
    queryFn: () => api.remoteBranchPlan(repo, remote, branch),
    enabled,
  });
}

/** 버전 상세(D-105). 버전은 바뀌지 않으므로 다시 받지 않는다. */
export function useCommitDetail(repo: string, oid: string | null) {
  return useQuery<Awaited<ReturnType<typeof api.commitDetail>>, ApiError>({
    queryKey: queryKeys.commitDetail(repo, oid ?? ""),
    queryFn: () => api.commitDetail(repo, oid ?? ""),
    enabled: oid !== null,
    staleTime: Infinity,
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
