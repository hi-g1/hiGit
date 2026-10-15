// 장면 좌표로 고르기(D-017: 클릭 판정은 DOM이 아니라 배치 결과로). Canvas로 바꿔도 그대로다.
import type { Scene } from "./scene";

/** (x, y)에서 가장 가까운 역의 id. 역 반지름 + `slop` 안에 없으면 null. */
export function hitTest(scene: Scene, x: number, y: number, slop = 6): string | null {
  let best: { id: string; d: number } | null = null;
  for (const s of scene.stations) {
    if (!s.id) continue;
    const d = Math.hypot(s.x - x, s.y - y);
    if (d <= s.r + slop && (!best || d < best.d)) best = { id: s.id, d };
  }
  return best?.id ?? null;
}

/** 세로 목록에서 y가 가리키는 행. */
export function rowAt(y: number, rowHeight: number, rows: number): number | null {
  const row = Math.floor(y / rowHeight);
  return row >= 0 && row < rows ? row : null;
}
