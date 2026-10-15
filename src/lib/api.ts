// 화면이 백엔드를 부르는 유일한 입구(D-010). 생성된 바인딩을 감싸고 오류를 ApiError로 바꾼다.
import { commands, type ErrorCode, type ErrorView, type LogQuery } from "./bindings";

export type {
  ActionMetaView,
  ActionResult,
  ChangeKind,
  ChangedFile,
  CommitRow,
  ErrorCode,
  FolderInfo,
  LogPage,
  LogQuery,
  RepoInfo,
  RiskView,
  StatusView,
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
  openFolder: (path: string) => call(commands.repoOpen(path)),
  listActions: () => call(commands.actionList()),
  previewAction: (folder: string, id: string, input: Record<string, string> = {}) =>
    call(commands.actionPreview(folder, id, input)),
  executeAction: (
    folder: string,
    id: string,
    input: Record<string, string> = {},
    previewToken: string | null = null,
  ) => call(commands.actionExecute(folder, id, input, previewToken)),
  status: (repo: string) => call(commands.queryStatus(repo)),
  log: (repo: string, query: LogQuery) => call(commands.queryLog(repo, query)),
};
