// 화면이 백엔드를 부르는 유일한 입구(D-010). 생성된 바인딩을 감싸고 오류를 ApiError로 바꾼다.
import {
  commands,
  type ActionInput,
  type DiffRequest,
  type ErrorCode,
  type ErrorView,
  type LogQuery,
  type OpenHow,
  type TrustChoice,
} from "./bindings";

export type {
  ActionInput,
  ActionMetaView,
  ActionResult,
  BranchListView,
  BranchNameCheck,
  BranchRow,
  BranchView,
  ChangeKind,
  ChangedFile,
  CommitDetailView,
  CommitFileView,
  CommitRow,
  DiffLineView,
  DiffRequest,
  ErrorCode,
  FileDiffView,
  HomeMapView,
  MapCommit,
  OpenHow,
  OtherOperation,
  PendingWorkView,
  ProjectOpenResult,
  ProjectSummaryView,
  RecentProject,
  RefLabelView,
  CommitStatView,
  HunkView,
  LineKindView,
  LogPage,
  LogScope,
  LogQuery,
  NameProblem,
  RepoInfo,
  RewriteBlocked,
  RewriteCardView,
  RewriteKindInput,
  RewritePlanView,
  RewriteStepInput,
  RiskView,
  MergeHintView,
  StashItem,
  RecoveryItem,
  RecoverySource,
  RecoveryView,
  StashListView,
  ConflictFileView,
  FileResolution,
  ConflictHunkView,
  ConflictSide,
  ConflictStateView,
  SideState,
  SecretCheckView,
  SecretFinding,
  SecretRule,
  StatusView,
  TrustChoice,
  UndoResult,
} from "./bindings";

/** 백엔드 오류. 화면은 `code`로 문구를 고르고 `detail`은 "자세히 보기"에 보여준다. */
export class ApiError extends Error {
  readonly code: ErrorCode;
  readonly detail: string | null;

  constructor(view: ErrorView) {
    super(view.code);
    this.name = "ApiError";
    this.code = view.code;
    this.detail = view.detail;
  }
}

function isErrorView(value: unknown): value is ErrorView {
  return typeof value === "object" && value !== null && "code" in value && "detail" in value;
}

export function toApiError(error: unknown): ApiError {
  if (error instanceof ApiError) return error;
  if (isErrorView(error)) return new ApiError(error);
  return new ApiError({ code: "internal", detail: String(error) });
}

async function call<T>(promise: Promise<T>): Promise<T> {
  try {
    return await promise;
  } catch (error) {
    throw toApiError(error);
  }
}

export const api = {
  appStatus: () => call(commands.appStatus()),
  /** Git 내려받기 페이지를 기본 브라우저로 연다. 주소는 백엔드가 정한다. */
  openGitDownload: () => call(commands.appOpenGitDownload()),
  /** 로그 폴더를 연다(D-078). 위치는 core가 정한다. */
  openLogFolder: () => call(commands.appOpenLogFolder()),
  /** 프로젝트 열기. 화면에서는 features/open-project/useOpenProject만 부른다(D-032). */
  openProject: (path: string, how: OpenHow) => call(commands.projectOpen(path, how)),
  /** 신뢰 확인에서 고른 결과로 연다(D-039). useOpenProject만 부른다. */
  trustProject: (path: string, choice: TrustChoice) => call(commands.projectTrust(path, choice)),
  /** 주인이 다른 폴더를 믿기로 하고 연다(D-039). useOpenProject만 부른다. */
  allowFolderOwner: (path: string) => call(commands.projectAllowOwner(path)),
  recentProjects: () => call(commands.recentList()),
  removeRecent: (path: string) => call(commands.recentRemove(path)),
  projectSummary: (repo: string) => call(commands.queryProjectSummary(repo)),
  listActions: () => call(commands.actionList()),
  previewAction: (folder: string, input: ActionInput) =>
    call(commands.actionPreview(folder, input)),
  executeAction: (folder: string, input: ActionInput, previewToken: string | null = null) =>
    call(commands.actionExecute(folder, input, previewToken)),
  /** 도는 원격 작업 취소, 멈춘 로컬 쓰기 강제로 멈추기(D-080). */
  cancelAction: (folder: string) => call(commands.actionCancel(folder)),
  /** 마지막 작업 되돌리기(Ctrl+Z)와 다시 하기(D-042). */
  undoLast: (folder: string) => call(commands.undoLast(folder)),
  redoLast: (folder: string) => call(commands.redoLast(folder)),
  status: (repo: string) => call(commands.queryStatus(repo)),
  log: (repo: string, query: LogQuery) => call(commands.queryLog(repo, query)),
  /** 버전별 바뀐 줄 수(노선도 선 굵기, D-089). 100개까지. */
  logStats: (repo: string, oids: string[]) => call(commands.queryLogStats(repo, oids)),
  /** 시작 화면 노선도(D-093). 최근 목록의 프로젝트도 열지 않고 읽는다. */
  routePreview: (repo: string, limit: number) => call(commands.queryRoutePreview(repo, limit)),
  rewritePlan: (repo: string, oids: string[]) => call(commands.queryRewritePlan(repo, oids)),
  commitDetail: (repo: string, oid: string) => call(commands.queryCommitDetail(repo, oid)),
  diff: (repo: string, request: DiffRequest) => call(commands.queryDiff(repo, request)),
  branches: (repo: string) => call(commands.queryBranches(repo)),
  homeMap: (repo: string) => call(commands.queryHomeMap(repo)),
  /** 충돌 해결 화면 상태(D-069). */
  conflictState: (repo: string) => call(commands.queryConflictState(repo)),
  /** 충돌 파일 하나의 고를 곳(D-086). */
  conflictFile: (repo: string, path: string) => call(commands.queryConflictFile(repo, path)),
  /** 작업 내역(D-085). */
  recovery: (repo: string) => call(commands.queryRecovery(repo)),
  /** 끝나지 않은 작업(D-082). */
  pendingWork: (repo: string) => call(commands.queryPendingWork(repo)),
  /** 임시 보관 목록(D-076). */
  stashes: (repo: string) => call(commands.queryStashes(repo)),
  /** 합치기 제안(D-048, D-071). */
  mergeHint: (repo: string) => call(commands.queryMergeHint(repo)),
  /** 버전 저장 전 비밀 파일 검사(D-077). 결과에는 경로·규칙·줄 번호만. */
  secretCheck: (repo: string, stageAll: boolean) => call(commands.querySecretCheck(repo, { stageAll })),
  checkBranchName: (repo: string, name: string) => call(commands.queryBranchName(repo, name)),
  watch: (repo: string) => call(commands.repoWatch(repo)),
  unwatch: () => call(commands.repoUnwatch()),
};
