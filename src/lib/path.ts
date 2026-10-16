// 저장소 안 경로 나누기. git이 주는 `/` 구분 경로다(Windows도 같다).

/** "docs/plans/010.md" → { name: "010.md", dir: "docs/plans" }. 최상위 파일은 dir이 빈 글자. */
export function splitPath(path: string): { name: string; dir: string } {
  const at = path.lastIndexOf("/");
  return at < 0 ? { name: path, dir: "" } : { name: path.slice(at + 1), dir: path.slice(0, at) };
}
