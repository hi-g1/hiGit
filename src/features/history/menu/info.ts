// 고른 역의 정보(D-092). 역 메뉴 항목이 이것만 보고 보일지·할 일을 정한다. 반복 3의 액션은
// `oid`를 입력에 담아 보내고 core가 그 해시를 다시 확인한다.
import type { RouteNode } from "../model";

export type StationInfo = {
  oid: string;
  shortOid: string;
  summary: string;
  author: string;
  /** unix 초 */
  time: number;
  parents: string[];
  isMerge: boolean;
  isRoot: boolean;
  isHead: boolean;
  /** 지금 브랜치의 선(지금 위치에서 첫 부모로 닿음) 위에 있다. */
  onHeadLine: boolean;
  /** 지금 위치의 조상이다(합쳐 들어온 곁가지 포함). */
  isAncestorOfHead: boolean;
  branches: string[];
  tags: string[];
  unpushed: boolean;
};

/** 지금 위치에서 모든 부모를 따라 닿는 노드. */
export function headAncestors(nodes: readonly RouteNode[]): ReadonlySet<string> {
  const byId = new Map(nodes.map((n) => [n.id, n]));
  const out = new Set<string>();
  const stack = nodes.filter((n) => n.isHead).map((n) => n.id);
  while (stack.length > 0) {
    const id = stack.pop()!;
    if (out.has(id)) continue;
    const node = byId.get(id);
    if (!node) continue;
    out.add(id);
    stack.push(...node.parents);
  }
  return out;
}

/** 버전 역의 정보. 접힌 구간·고친 파일 역은 메뉴가 없어 null. */
export function stationInfo(
  node: RouteNode,
  onHeadLine: ReadonlySet<string>,
  ancestors: ReadonlySet<string>,
): StationInfo | null {
  if (node.kind !== "commit") return null;
  return {
    oid: node.id,
    shortOid: node.shortOid,
    summary: node.summary,
    author: node.author,
    time: node.time,
    parents: node.parents,
    isMerge: node.parents.length > 1,
    isRoot: node.parents.length === 0,
    isHead: node.isHead,
    onHeadLine: onHeadLine.has(node.id),
    isAncestorOfHead: ancestors.has(node.id),
    branches: node.branches,
    tags: node.tags,
    unpushed: node.unpushed,
  };
}
