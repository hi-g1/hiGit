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
  ChangeKind,
  ChangedFile,
  CommitRow,
  DiffLineView,
  DiffRequest,
  ErrorCode,
  FileDiffView,
  OpenHow,
  ProjectOpenResult,
  ProjectSummaryView,
  RecentProject,
  HunkView,
  LineKindView,
  LogPage,
  LogQuery,
  NameProblem,
  RepoInfo,
  RiskView,
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
  /** 마지막 작업 되돌리기(Ctrl+Z)와 다시 하기(D-042). */
  undoLast: (folder: string) => call(commands.undoLast(folder)),
  redoLast: (folder: string) => call(commands.redoLast(folder)),
  status: (repo: string) => call(commands.queryStatus(repo)),
  log: (repo: string, query: LogQuery) => call(commands.queryLog(repo, query)),
  diff: (repo: string, request: DiffRequest) => call(commands.queryDiff(repo, request)),
  branches: (repo: string) => call(commands.queryBranches(repo)),
  checkBranchName: (repo: string, name: string) => call(commands.queryBranchName(repo, name)),
  watch: (repo: string) => call(commands.repoWatch(repo)),
  unwatch: () => call(commands.repoUnwatch()),
};
