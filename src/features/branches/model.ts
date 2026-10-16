// 브랜치 목록 모델(D-047, r-branch.png). 화면 없이 시험할 수 있게 순수 함수로 둔다.
import type { BranchListView, BranchRow } from "../../lib/api";

export const RECENT_COUNT = 5;

export type ListItem = {
  /** 로컬 브랜치 이름. 원격에만 있으면 원격 쪽 브랜치 이름(origin/ 뺀 것). */
  name: string;
  remote: string | null;
  row: BranchRow;
};

export type Sections = {
  /** 고정한 브랜치(D-203). 지금 브랜치는 "지금"에 두고 여기서 뺀다. */
  pinned: ListItem[];
  current: ListItem | null;
  recent: ListItem[];
  more: ListItem[];
};

/** 원격에만 있는 브랜치(같은 이름의 로컬 브랜치가 없는 것). */
function remoteOnly(view: BranchListView): ListItem[] {
  const locals = new Set(view.local.map((b) => b.name));
  return view.remote
    .filter((b) => b.remote !== null)
    .map((b) => ({ name: b.name.slice(b.remote!.length + 1), remote: b.remote, row: b }))
    .filter((i) => i.name !== "HEAD" && !locals.has(i.name));
}

/** "고정"(있는 로컬 브랜치만), "지금", "최근에 쓴 브랜치"(마지막 버전 순 5개), 나머지. */
export function sections(view: BranchListView, pinnedNames: readonly string[] = []): Sections {
  const head = view.local.find((b) => b.isHead) ?? null;
  const pinnedSet = new Set(pinnedNames);
  const pinned = view.local
    .filter((b) => !b.isHead && pinnedSet.has(b.name))
    .sort((a, b) => pinnedNames.indexOf(a.name) - pinnedNames.indexOf(b.name))
    .map((row) => ({ name: row.name, remote: null, row }));
  const others = view.local
    .filter((b) => !b.isHead && !pinnedSet.has(b.name))
    .sort((a, b) => b.lastAt - a.lastAt)
    .map((row) => ({ name: row.name, remote: null, row }));
  return {
    pinned,
    current: head && { name: head.name, remote: null, row: head },
    recent: others.slice(0, RECENT_COUNT),
    more: [...others.slice(RECENT_COUNT), ...remoteOnly(view)],
  };
}

/** 찾기: 이름에 글자가 들어 있는 브랜치(대소문자 무시). 지금 브랜치도 포함한다. */
export function search(view: BranchListView, query: string): ListItem[] {
  const q = query.trim().toLowerCase();
  const all: ListItem[] = [
    ...view.local.map((row) => ({ name: row.name, remote: null, row })),
    ...remoteOnly(view),
  ];
  return q ? all.filter((i) => i.name.toLowerCase().includes(q)) : all;
}

/** 찾기 칸 글자로 새 브랜치를 만들 수 있는지(같은 이름이 없을 때, 대소문자 무시). */
export function canOfferCreate(view: BranchListView, query: string): boolean {
  const q = query.trim().toLowerCase();
  return q !== "" && !view.local.some((b) => b.name.toLowerCase() === q);
}

export type RightText =
  | { kind: "parkBlocked"; count: number }
  | { kind: "parked"; count: number }
  | { kind: "newVersions"; count: number }
  | { kind: "author"; author: string; at: number };

/** 줄 오른쪽 글자 우선순위(D-049): 맡긴 파일(겹침) > 맡긴 파일 > upstream의 새 버전 > 작성자·날짜. */
export function rightText(row: BranchRow): RightText {
  if (row.parkBlocked) return { kind: "parkBlocked", count: row.parkedCount };
  if (row.parkedCount > 0) return { kind: "parked", count: row.parkedCount };
  if (row.behind > 0) return { kind: "newVersions", count: row.behind };
  return { kind: "author", author: row.lastAuthor, at: row.lastAt };
}
