// 노선도 확대·축소(D-087). 받은 버전을 단계에 따라 접는다: 남기지 않는 버전은 "버전 N개" 한 줄로.
// 백엔드는 단계를 모른다. DOM 없음.
import type { RouteNode } from "../model";

export type Zoom = "all" | "key" | "tags";
export const ZOOMS: readonly Zoom[] = ["all", "key", "tags"];

/** 접힌 구간이 기억하는 버전 해시 수(줄 수 합산). 넘으면 앞부분만. */
const MAX_STAT_OIDS = 500;

/**
 * 단계에 따라 접는다. 남기는 것: 주요 버전 단계는 합치기·갈라진 곳·이름표(브랜치·태그)·지금 위치·첫
 * 버전, 태그만 단계는 태그·브랜치 끝·지금 위치·첫 버전. 모든 단계에서 부모가 받은 범위 밖인 버전(쪽
 * 끝)은 남긴다. 화면 순서에서 이어지고 첫 부모로 이어진 숨김 버전들이 접힌 구간 하나가 된다. 그래서
 * 위상 순서가 그대로이고 선이 끊기지 않는다. `expanded`에 든 구간은 펼친다.
 */
export function simplify(nodes: readonly RouteNode[], zoom: Zoom, expanded: ReadonlySet<string> = new Set()): RouteNode[] {
  if (zoom === "all" || nodes.length === 0) return [...nodes];
  const ids = new Set(nodes.map((n) => n.id));
  const children = new Map<string, number>();
  for (const n of nodes) for (const p of n.parents) children.set(p, (children.get(p) ?? 0) + 1);
  const keep = (n: RouteNode): boolean => {
    if (n.kind !== "commit" || n.isHead || n.parents.length === 0) return true;
    if (n.parents.some((p) => !ids.has(p))) return true;
    if (zoom === "tags") return n.tags.length > 0 || n.branches.length > 0;
    return n.parents.length > 1 || (children.get(n.id) ?? 0) > 1 || n.tags.length > 0 || n.branches.length > 0;
  };

  // 화면 순서에서 이어진 숨김 구간을 나눈다(다음 노드가 지금 끝의 첫 부모일 때만 이어 간다).
  const runs: RouteNode[][] = [];
  const out: (RouteNode | RouteNode[])[] = [];
  let run: RouteNode[] | null = null;
  for (const n of nodes) {
    if (keep(n)) {
      run = null;
      out.push(n);
      continue;
    }
    const last = run?.[run.length - 1];
    if (run && last && last.parents[0] === n.id) {
      run.push(n);
    } else {
      run = [n];
      runs.push(run);
      out.push(run);
    }
  }

  // 펼친 구간은 그대로 둔다.
  const foldOf = new Map<string, string>();
  const foldIds = new Map<RouteNode[], string>();
  for (const r of runs) {
    const id = `${r[0]!.id}>${r[r.length - 1]!.id}`;
    if (expanded.has(id)) continue;
    foldIds.set(r, id);
    for (const m of r) foldOf.set(m.id, id);
  }
  const mapParent = (p: string) => foldOf.get(p) ?? p;
  const remap = (parents: readonly string[], self?: string): string[] => {
    const seen = new Set<string>();
    const result: string[] = [];
    for (const p of parents) {
      const q = mapParent(p);
      if (q === self || seen.has(q)) continue;
      seen.add(q);
      result.push(q);
    }
    return result;
  };

  const result: RouteNode[] = [];
  for (const item of out) {
    if (!Array.isArray(item)) {
      result.push({ ...item, parents: remap(item.parents) });
      continue;
    }
    const id = foldIds.get(item);
    if (!id) {
      for (const m of item) result.push({ ...m, parents: remap(m.parents) });
      continue;
    }
    const members = new Set(item.map((m) => m.id));
    const external = item.flatMap((m) => m.parents.filter((p) => !members.has(p)));
    // 사슬 끝의 첫 부모를 맨 앞에 둔다(레인이 이어지는 쪽).
    const chainParent = item[item.length - 1]!.parents[0];
    const ordered = chainParent ? [chainParent, ...external.filter((p) => p !== chainParent)] : external;
    result.push({
      id,
      parents: remap(ordered, id),
      kind: "fold",
      shortOid: "",
      summary: "",
      author: "",
      time: 0,
      branches: [],
      tags: [],
      isHead: false,
      unpushed: item.some((m) => m.unpushed),
      count: item.length,
      statOids: item.slice(0, MAX_STAT_OIDS).map((m) => m.id),
    });
  }
  return result;
}
