// 노선도 레인 배치(D-090, D-091). 위상 순서(최신이 위)의 노드에 레인을 정하고 노드 사이 선을
// 만든다. 버전 기록·홈·시작 화면이 함께 쓴다. DOM 없음.
import type { RouteNode } from "../model";
import type { ColorKey } from "./scene";

export type LaneOptions = {
  /** 0번 레인의 끝점(보통 기본 브랜치 끝). 나올 때까지 0번 칸을 비워 둔다. */
  trunkId?: string | null;
  /** 0번 레인의 이름(기본 브랜치). */
  trunkName?: string | null;
  /** 지금 브랜치. 레인 이름을 고를 때 먼저 본다. */
  headBranch?: string | null;
  /** 그리는 레인 수 상한(홈은 2). 넘는 레인은 마지막 레인에 겹쳐 그린다. */
  maxLanes?: number;
};

export type LaneNode = { id: string; row: number; lane: number; color: ColorKey; isMerge: boolean };

/**
 * 노드 사이 선. `travel` 레인을 따라 가고, 위 끝(자식)이나 아래 끝(부모)에서 레인이 다르면 그 행
 * 안에서 꺾는다. `toRow`가 null이면 받은 범위 밖의 부모라 아래 끝까지 이어진다.
 */
export type LaneEdge = {
  from: string;
  to: string;
  fromRow: number;
  toRow: number | null;
  fromLane: number;
  toLane: number;
  travel: number;
  color: ColorKey;
};

export type LaneLayout = {
  nodes: LaneNode[];
  edges: LaneEdge[];
  laneCount: number;
  /** 지금 위치에서 부모를 따라 닿는 노드(지금 브랜치 선 굵기). */
  onHeadLine: ReadonlySet<string>;
};

/** 이름 있는 레인은 `b:<브랜치>`, 이름 없는 레인(지운 브랜치)은 `u:<레인을 여는 버전>`(D-091). */
export const branchKey = (name: string): ColorKey => `b:${name}`;
export const unnamedKey = (id: string): ColorKey => `u:${id}`;

/** 레인 이름: 지금 브랜치 > 기본 브랜치 > 이름순 첫째. */
function laneName(node: RouteNode, opts: LaneOptions): string | null {
  if (node.branches.length === 0) return null;
  if (opts.headBranch && node.branches.includes(opts.headBranch)) return opts.headBranch;
  if (opts.trunkName && node.branches.includes(opts.trunkName)) return opts.trunkName;
  return [...node.branches].sort()[0] ?? null;
}

type Pending = { from: string; fromRow: number; fromSlot: number; slot: number };

export function assignLanes(nodes: readonly RouteNode[], opts: LaneOptions = {}): LaneLayout {
  const cap = Math.max(1, opts.maxLanes ?? Number.POSITIVE_INFINITY);
  const draw = (slot: number) => Math.min(slot, cap - 1);
  const ids = new Set(nodes.map((n) => n.id));
  const trunkId = opts.trunkId && ids.has(opts.trunkId) ? opts.trunkId : null;

  // 칸마다 기다리는 노드 id. 칸을 새로 열 때마다 레인 인스턴스가 새로 생기고 색은 그 레인에 처음
  // 놓이는 노드가 정한다.
  const slots: (string | null)[] = [];
  const slotInstance: number[] = [];
  const instanceColor: (ColorKey | null)[] = [];
  const open = (slot: number, waitFor: string, color: ColorKey | null) => {
    slots[slot] = waitFor;
    slotInstance[slot] = instanceColor.length;
    instanceColor.push(color);
  };
  const freeSlot = (): number => {
    const free = slots.findIndex((s) => s === null);
    return free >= 0 ? free : slots.length;
  };
  if (trunkId) open(0, trunkId, opts.trunkName ? branchKey(opts.trunkName) : null);

  const pending: Pending[] = [];
  const placed = new Map<string, { row: number; slot: number; instance: number }>();
  const raw: (Omit<LaneEdge, "color" | "fromLane" | "toLane" | "travel"> & {
    fromSlot: number;
    toSlot: number;
    travelSlot: number;
    instance: number;
  })[] = [];
  let maxSlots = slots.length;

  nodes.forEach((node, row) => {
    const waiting = slots.flatMap((s, i) => (s === node.id ? [i] : []));
    let slot: number;
    if (waiting.length > 0) {
      slot = Math.min(...waiting);
    } else {
      slot = freeSlot();
      open(slot, node.id, null);
    }
    const instance = slotInstance[slot]!;
    if (instanceColor[instance] === null) {
      const name = laneName(node, opts);
      instanceColor[instance] = name ? branchKey(name) : unnamedKey(node.id);
    }
    // 이 노드를 기다리던 선을 닫는다. 가장 왼쪽이 아닌 칸은 여기서 합류하고 비운다.
    for (let i = pending.length - 1; i >= 0; i--) {
      const p = pending[i]!;
      if (slots[p.slot] !== node.id) continue;
      raw.push({
        from: p.from,
        to: node.id,
        fromRow: p.fromRow,
        toRow: row,
        fromSlot: p.fromSlot,
        toSlot: slot,
        travelSlot: p.slot,
        instance: slotInstance[p.slot]!,
      });
      pending.splice(i, 1);
    }
    for (const w of waiting) slots[w] = null;
    placed.set(node.id, { row, slot, instance });

    node.parents.forEach((parent, i) => {
      const above = placed.get(parent);
      if (above) {
        // 부모가 이미 위에 있다(어긋난 입력). 바로 잇는다.
        raw.push({
          from: node.id,
          to: parent,
          fromRow: row,
          toRow: above.row,
          fromSlot: slot,
          toSlot: above.slot,
          travelSlot: above.slot,
          instance,
        });
        return;
      }
      let travel: number;
      if (i === 0) {
        // 첫 부모는 이 레인이 이어받는다. 다른 칸이 같은 부모를 기다려도 부모 행에서 합류한다.
        travel = slot;
        slots[slot] = parent;
      } else {
        const already = slots.indexOf(parent);
        if (already >= 0) {
          travel = already;
        } else {
          travel = freeSlot();
          open(travel, parent, null);
        }
      }
      pending.push({ from: node.id, fromRow: row, fromSlot: slot, slot: travel });
    });
    maxSlots = Math.max(maxSlots, slots.length);
  });

  // 받은 범위 밖의 부모로 가는 선은 아래 끝까지.
  for (const p of pending) {
    const to = slots[p.slot] ?? "";
    const instance = slotInstance[p.slot]!;
    if (instanceColor[instance] === null) instanceColor[instance] = unnamedKey(to);
    raw.push({
      from: p.from,
      to,
      fromRow: p.fromRow,
      toRow: null,
      fromSlot: p.fromSlot,
      toSlot: p.slot,
      travelSlot: p.slot,
      instance,
    });
  }

  const colorOf = (instance: number): ColorKey => instanceColor[instance] ?? unnamedKey(String(instance));
  return {
    nodes: nodes.map((n) => {
      const p = placed.get(n.id)!;
      return { id: n.id, row: p.row, lane: draw(p.slot), color: colorOf(p.instance), isMerge: n.parents.length > 1 };
    }),
    edges: raw.map(({ fromSlot, toSlot, travelSlot, instance, ...e }) => ({
      ...e,
      fromLane: draw(fromSlot),
      toLane: draw(toSlot),
      travel: draw(travelSlot),
      color: colorOf(instance),
    })),
    laneCount: Math.max(1, Math.min(maxSlots, cap)),
    onHeadLine: headLine(nodes),
  };
}

/** 지금 위치에서 부모를 따라 닿는 노드. */
function headLine(nodes: readonly RouteNode[]): ReadonlySet<string> {
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
