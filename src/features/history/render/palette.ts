// 노선도 색 키 → CSS 색(D-091, D-031). 선과 점에만 쓴다.
import { branchColor } from "../../branches/color";
import type { ColorKey } from "../layout/scene";
import type { Palette } from "./Renderer";

/** 이름 없는 레인(지운 브랜치): 레인을 여는 버전 해시로 --graph-2~6 중 하나. */
function unnamedColor(id: string): string {
  let hash = 0;
  for (const ch of id) hash = (hash * 31 + (ch.codePointAt(0) ?? 0)) >>> 0;
  return `var(--graph-${2 + (hash % 5)})`;
}

/** 브랜치 색: `b:<이름>`은 branchColor, `u:<버전>`은 해시 색. */
export function branchPalette(defaultBranch: string | null): Palette {
  return (key: ColorKey) => {
    if (key.startsWith("b:")) return branchColor(key.slice(2), defaultBranch);
    if (key.startsWith("u:")) return unnamedColor(key.slice(2));
    return "var(--text-muted)";
  };
}

/** 회색(시작 화면 작은 노선도): 기본 브랜치 줄은 --border-strong, 나머지는 --text-muted. */
export function monoPalette(defaultBranch: string | null): Palette {
  return (key: ColorKey) =>
    defaultBranch !== null && key === `b:${defaultBranch}` ? "var(--border-strong)" : "var(--text-muted)";
}
