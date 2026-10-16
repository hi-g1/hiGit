// 기본 앱으로 열 수 있는지 미리 본다(D-216). 차단표는 core 상수 그대로이고 core가 실행 때 다시 본다.
import { OPEN_BLOCKED_EXTENSIONS } from "./bindings";

const BLOCKED = new Set<string>(OPEN_BLOCKED_EXTENSIONS);

/** 마지막 확장자가 차단표에 있는지(대소문자 무시). core의 `blocked_extension`과 같은 규칙. */
export function blockedExtension(path: string): boolean {
  const name = path.split(/[/\\]/).pop() ?? path;
  const dot = name.lastIndexOf(".");
  return dot >= 0 && BLOCKED.has(name.slice(dot + 1).toLowerCase());
}

/** 메뉴 항목을 끌 까닭. 열 수 있으면 null. */
export type OpenBlock = "deleted" | "runnable" | null;

export function openBlock(path: string, deleted: boolean): OpenBlock {
  if (deleted) return "deleted";
  return blockedExtension(path) ? "runnable" : null;
}
