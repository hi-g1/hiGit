// 버전 이름 제안(D-044). "화면.tsx 외 2개 고침"처럼 파일 이름과 개수로 만든다.
import type { ChangedFile } from "../../lib/api";
import type { Messages } from "../../messages";

export type SuggestKind = "update" | "add" | "delete" | "rename";

function kindOf(file: ChangedFile): SuggestKind {
  switch (file.change) {
    case "added":
    case "untracked":
      return "add";
    case "deleted":
      return "delete";
    case "renamed":
      return "rename";
    default:
      return "update";
  }
}

function baseName(path: string): string {
  return path.split("/").pop() ?? path;
}

/** 이번 버전에 들어갈 파일: 넣은 파일이 있으면 그것, 없으면 고친 파일 모두(D-062). */
export function filesToSave(files: ChangedFile[]): ChangedFile[] {
  const staged = files.filter((f) => f.staged);
  return staged.length > 0 ? staged : files;
}

/** 제안 이름. 파일이 없으면 빈 문자열. */
export function suggestName(files: ChangedFile[], m: Messages): string {
  const target = filesToSave(files);
  const first = target[0];
  if (!first) return "";
  if (target.length === 1) {
    const kind = kindOf(first);
    const name = kind === "rename" && first.origPath ? baseName(first.origPath) : baseName(first.path);
    return m.suggest.one(kind, name);
  }
  const kinds = new Set(target.map(kindOf));
  const kind: SuggestKind = kinds.size === 1 && (kinds.has("add") || kinds.has("delete")) ? [...kinds][0]! : "update";
  return m.suggest.many(kind, baseName(first.path), target.length - 1);
}
