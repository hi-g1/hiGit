// 브랜치 색(design.md). 기본 브랜치는 --graph-1, 나머지는 이름 해시로 --graph-2~6.
// 상단 바 점, 브랜치 목록 점, 홈 노선도가 같은 함수를 쓴다. 글자에는 쓰지 않는다(D-031).

export function branchColor(name: string, defaultBranch: string | null): string {
  if (defaultBranch !== null && name === defaultBranch) return "var(--graph-1)";
  let hash = 0;
  for (const ch of name) hash = (hash * 31 + (ch.codePointAt(0) ?? 0)) >>> 0;
  return `var(--graph-${2 + (hash % 5)})`;
}
