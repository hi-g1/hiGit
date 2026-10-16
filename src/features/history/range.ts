// 여러 역 선택(계획 008 E27, D-106): 지금 브랜치 첫 부모 선 위의 연속 구간. DOM 없음.
import type { Zoom } from "./layout/simplify";
import type { RouteNode } from "./model";

/** 기록 정리에서 한 번에 고르는 버전 수(core의 상한과 같다). */
export const MAX_RANGE = 50;

/** 지금 위치에서 첫 부모를 따라간 노드 id(새 것부터). 받은 범위를 벗어나면 거기서 끝. */
export function headLine(nodes: readonly RouteNode[]): string[] {
  const byId = new Map(nodes.map((n) => [n.id, n]));
  const out: string[] = [];
  let node = nodes.find((n) => n.isHead);
  while (node && !out.includes(node.id)) {
    out.push(node.id);
    const parent = node.parents[0];
    node = parent === undefined ? undefined : byId.get(parent);
  }
  return out;
}

/**
 * 범위의 움직이는 끝을 선 위의 이웃으로 옮긴다(곁가지 행은 건너뛴다). "next"는 아래(옛 쪽). 끝이거나
 * 선 위가 아니면 그대로.
 */
export function extend(line: readonly string[], active: string, dir: "next" | "prev"): string {
  const i = line.indexOf(active);
  if (i < 0) return active;
  const j = dir === "next" ? i + 1 : i - 1;
  return line[j] ?? active;
}

/** 두 끝 사이의 선 위 id(새 것부터). 둘 중 하나라도 선 위가 아니면 빈 목록. */
export function rangeIds(line: readonly string[], anchor: string, active: string): string[] {
  const a = line.indexOf(anchor);
  const b = line.indexOf(active);
  if (a < 0 || b < 0) return [];
  return line.slice(Math.min(a, b), Math.max(a, b) + 1);
}

/** 기록 정리를 열 수 있는 구간인지: 모든 버전 보기, 2~50개, 모두 버전(접힘 아님)이고 합친 버전 없음. */
export function eligible(ids: readonly string[], nodes: readonly RouteNode[], zoom: Zoom): boolean {
  if (zoom !== "all" || ids.length < 2 || ids.length > MAX_RANGE) return false;
  const byId = new Map(nodes.map((n) => [n.id, n]));
  return ids.every((id) => {
    const node = byId.get(id);
    return node !== undefined && node.kind === "commit" && node.parents.length === 1;
  });
}
