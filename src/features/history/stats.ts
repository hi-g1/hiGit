// 노선도 선 굵기용 줄 수를 보이는 버전만 늦게 받는다(D-089, D-096).
import { useEffect, useRef, useState } from "react";
import { api, type CommitStatView } from "../../lib/api";
import type { LineStats } from "./layout/route";

/** 한 번에 묻는 버전 수(core 상한과 같다). */
export const STATS_BATCH = 100;
/** 스크롤이 멈추기를 기다리는 시간. */
export const STATS_DELAY_MS = 150;
/** 기억하는 버전 수. 넘으면 오래된 것부터 버린다. */
const MAX_CACHED = 20_000;

type Cache = { repo: string; map: ReadonlyMap<string, number | null> };

/** 받은 결과를 더한 새 표. 결과에 없는 해시는 "모름"(null)으로 두어 다시 묻지 않는다. */
function merged(map: ReadonlyMap<string, number | null>, asked: readonly string[], stats: readonly CommitStatView[]) {
  const next = new Map(map);
  for (const oid of asked) next.set(oid, null);
  for (const s of stats) next.set(s.oid, s.added + s.removed);
  const over = next.size - MAX_CACHED;
  if (over > 0) for (const oid of [...next.keys()].slice(0, over)) next.delete(oid);
  return next;
}

/**
 * `wanted`(보이는 버전 해시) 중 아직 모르는 것을 150ms 뒤 100개씩 하나씩 묻는다. 버전의 줄 수는
 * 바뀌지 않으므로 다시 묻지 않는다.
 */
export function useCommitStats(repo: string, wanted: readonly string[]): LineStats {
  const [cache, setCache] = useState<Cache>(() => ({ repo, map: new Map() }));
  if (cache.repo !== repo) setCache({ repo, map: new Map() });
  const map = cache.map;
  const busy = useRef(false);
  const key = wanted.join(",");

  useEffect(() => {
    const missing = key.split(",").filter((oid) => oid && !map.has(oid)).slice(0, STATS_BATCH);
    if (missing.length === 0) return;
    const timer = setTimeout(() => {
      if (busy.current) return;
      busy.current = true;
      // 답은 화면이 바뀌어도 그 저장소의 값이라 버리지 않는다. 표가 바뀌면 남은 것을 다시 묻는다.
      const done = (stats: readonly CommitStatView[]) => {
        busy.current = false;
        setCache((c) => (c.repo === repo ? { repo, map: merged(c.map, missing, stats) } : c));
      };
      api.logStats(repo, missing).then(done, () => done([]));
    }, STATS_DELAY_MS);
    return () => clearTimeout(timer);
  }, [repo, key, map]);

  return map;
}
