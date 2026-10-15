// 버전 기록 화면 모델(docs/architecture.md "버전 기록 그리기 계층"). 받은 쪽을 노선도 노드로 바꾼다.
// 배치 계산·그리기는 여기 두지 않는다.
import type { LogPage } from "../../lib/api";

/** 노선도 노드(D-090). 버전 하나, 접힌 구간("버전 N개"), 또는 홈의 "고친 파일"(아직 버전 아님). */
export type RouteNode = {
  id: string;
  /** 부모 id. 위상 순서에서 부모는 늘 아래(뒤)에 온다. */
  parents: string[];
  kind: "commit" | "fold" | "open";
  shortOid: string;
  summary: string;
  author: string;
  /** unix 초. 버전이 아니면 0. */
  time: number;
  branches: string[];
  tags: string[];
  isHead: boolean;
  unpushed: boolean;
  /** 접힌 구간이 담은 버전 수. */
  count: number;
  /** 접힌 구간이 담은 버전 해시(줄 수 합산용, 최대 500개). */
  statOids: string[];
};

export function commitNode(id: string, parents: string[], over: Partial<RouteNode> = {}): RouteNode {
  return {
    id,
    parents,
    kind: "commit",
    shortOid: id.slice(0, 7),
    summary: "",
    author: "",
    time: 0,
    branches: [],
    tags: [],
    isHead: false,
    unpushed: false,
    count: 1,
    statOids: [id],
    ...over,
  };
}

/** 받은 쪽들을 이어 노드로. 같은 해시는 한 번만. 지금 위치·안 올림은 첫 쪽이 준다. */
export function toRouteNodes(pages: readonly LogPage[]): RouteNode[] {
  const first = pages[0];
  const unpushed = new Set(first?.unpushed ?? []);
  const branches = new Map<string, string[]>();
  const tags = new Map<string, string[]>();
  for (const page of pages) {
    for (const l of page.labels) {
      const into = l.kind === "branch" ? branches : tags;
      const names = into.get(l.oid) ?? [];
      if (!names.includes(l.name)) into.set(l.oid, [...names, l.name]);
    }
  }
  const seen = new Set<string>();
  const nodes: RouteNode[] = [];
  for (const page of pages) {
    for (const c of page.commits) {
      if (seen.has(c.oid)) continue;
      seen.add(c.oid);
      nodes.push(
        commitNode(c.oid, c.parents, {
          shortOid: c.shortOid,
          summary: c.summary,
          author: c.authorName,
          time: c.authorTime,
          branches: branches.get(c.oid) ?? [],
          tags: tags.get(c.oid) ?? [],
          isHead: c.oid === first?.head,
          unpushed: unpushed.has(c.oid),
        }),
      );
    }
  }
  return nodes;
}
