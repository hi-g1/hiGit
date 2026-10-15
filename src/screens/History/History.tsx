import { useQueryClient } from "@tanstack/react-query";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { buildRoute, ROW_H, verticalScene } from "../../features/history/layout/route";
import { simplify, ZOOMS, type Zoom } from "../../features/history/layout/simplify";
import { headAncestors, stationInfo } from "../../features/history/menu/info";
import type { MenuCtx, MenuEnv } from "../../features/history/menu/items";
import { StationMenu } from "../../features/history/menu/StationMenu";
import { toRouteNodes } from "../../features/history/model";
import { branchPalette } from "../../features/history/render/palette";
import { RouteList } from "../../features/history/RouteList";
import { useCommitStats } from "../../features/history/stats";
import { HISTORY_MAX, useLogPages, useStatus } from "../../lib/queries";
import { useShortcut } from "../../lib/useShortcut";
import { useWheelZoom } from "../../lib/wheelZoom";
import { useMessages } from "../../messages";
import { useHistoryStore } from "../../state/historyStore";
import { useRepoStore } from "../../state/repoStore";
import { EmptyState } from "../../ui/EmptyState";
import { ErrorNotice } from "../../ui/ErrorNotice";
import { Term, TermTooltip } from "../../ui/Term";
import { useToastStore } from "../../ui/toastStore";
import styles from "./History.module.css";

/** 접혀서 행이 모자랄 때 저절로 이어 받는 쪽 수. */
const AUTO_PAGES = 10;
/** "지금 여기"를 찾아 이어 받는 쪽 수. */
const SEEK_PAGES = 25;

/**
 * 버전 기록 화면(D-058, D-087~D-095): 창 전체 폭 노선도(r-log.png). 머리에 범위 전환과 확대·축소,
 * "지금 여기". Esc나 상단 바 "버전 기록"을 다시 누르면 홈으로 돌아간다.
 */
export function History({ repo, onDone }: { repo: string; onDone: (text: string) => void }) {
  const m = useMessages();
  const queryClient = useQueryClient();
  const show = useToastStore((s) => s.show);
  const changedFiles = useStatus(repo).data?.files.length ?? 0;
  const setView = useRepoStore((s) => s.setView);
  const { selectedId, zoom, scope, expanded, select, setZoom, setScope, expand } = useHistoryStore();
  const log = useLogPages(repo, scope);
  const { hasNextPage, isFetchingNextPage, fetchNextPage } = log;
  const pages = log.data?.pages;
  const first = pages?.[0];
  const [now] = useState(() => Date.now());
  const box = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape" && !e.defaultPrevented) setView("home");
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [setView]);

  const nodes = useMemo(() => toRouteNodes(pages ?? []), [pages]);
  const shown = useMemo(() => simplify(nodes, zoom, expanded), [nodes, zoom, expanded]);
  const route = useMemo(
    () =>
      buildRoute(shown, {
        trunkId: first?.defaultTip ?? null,
        trunkName: first?.defaultBranch ?? null,
        headBranch: first?.currentBranch ?? null,
      }),
    [shown, first?.defaultTip, first?.defaultBranch, first?.currentBranch],
  );
  const [windowRows, setWindowRows] = useState<[number, number]>([0, 0]);
  const onWindow = useCallback((a: number, b: number) => setWindowRows([a, b]), []);
  const wanted = useMemo(
    () => shown.slice(windowRows[0], windowRows[1]).flatMap((n) => (n.kind === "open" ? [] : n.statOids)),
    [shown, windowRows],
  );
  const stats = useCommitStats(repo, wanted);
  const scene = useMemo(() => verticalScene(route, stats), [route, stats]);
  const palette = useMemo(() => branchPalette(first?.defaultBranch ?? null), [first?.defaultBranch]);
  const received = nodes.length;

  const more = useCallback(() => {
    if (hasNextPage && !isFetchingNextPage) void fetchNextPage();
  }, [hasNextPage, isFetchingNextPage, fetchNextPage]);

  // 접혀서 행이 화면보다 적으면 몇 쪽 더 받아 둔다.
  const auto = useRef(0);
  useEffect(() => {
    auto.current = 0;
  }, [zoom, scope]);
  useEffect(() => {
    if (shown.length * ROW_H < 2 * 600 && hasNextPage && !isFetchingNextPage && auto.current < AUTO_PAGES) {
      auto.current += 1;
      void fetchNextPage();
    }
  }, [shown.length, hasNextPage, isFetchingNextPage, fetchNextPage]);

  // "지금 여기": 지금 위치가 아직 안 왔으면 이어 받아 찾는다(최대 25쪽).
  const seeking = useRef(0);
  const head = shown.find((n) => n.isHead);
  const goHere = () => {
    if (head) {
      select(head.id);
      return;
    }
    seeking.current = 1;
    more();
  };
  useEffect(() => {
    if (seeking.current === 0) return;
    if (head) {
      seeking.current = 0;
      select(head.id);
    } else if (hasNextPage && seeking.current <= SEEK_PAGES) {
      if (!isFetchingNextPage) {
        seeking.current += 1;
        void fetchNextPage();
      }
    } else {
      seeking.current = 0;
    }
    // 쪽이 올 때마다(받은 수가 바뀔 때) 다시 본다.
  }, [head, received, hasNextPage, isFetchingNextPage, fetchNextPage, select]);

  const changeZoom = useCallback(
    (dir: "in" | "out") => {
      const i = ZOOMS.indexOf(zoom) + (dir === "in" ? -1 : 1);
      const next = ZOOMS[Math.max(0, Math.min(ZOOMS.length - 1, i))]!;
      if (next !== zoom) setZoom(next);
    },
    [zoom, setZoom],
  );
  useShortcut("zoomIn", () => changeZoom("in"));
  useShortcut("zoomOut", () => changeZoom("out"));
  useWheelZoom(box, changeZoom);

  // 역 메뉴(D-092): 역을 누르거나 Enter로 연다. 같은 역을 다시 누르면 닫는다.
  const [menuOpen, setMenuOpen] = useState(false);
  const listEl = useRef<HTMLDivElement | null>(null);
  const onActivate = useCallback(
    (id: string) => {
      const node = shown.find((n) => n.id === id);
      if (node?.kind === "fold") {
        expand(id);
        setMenuOpen(false);
      } else if (node?.kind === "commit") {
        setMenuOpen((open) => !(open && id === selectedId));
      }
    },
    [shown, expand, selectedId],
  );
  const ancestors = useMemo(() => headAncestors(shown), [shown]);
  const selectedRow = selectedId === null ? -1 : shown.findIndex((n) => n.id === selectedId);
  const selectedNode = selectedRow >= 0 ? shown[selectedRow] : undefined;
  const info = selectedNode ? stationInfo(selectedNode, route.layout.onHeadLine, ancestors) : null;
  const anchor = {
    x: (scene.stations.find((s) => s.id === selectedId)?.x ?? 0) + 16,
    y: selectedRow * ROW_H + ROW_H / 2,
  };
  const env: MenuEnv = { repo, currentBranch: first?.currentBranch ?? null, changedFiles, merging: false };
  const ctx: MenuCtx = { repo, toast: show, queryClient, onDone, m };

  const zoomName = (z: Zoom) =>
    z === "tags" ? (
      <TermTooltip id="tag">
        <span>{m.zoomNames.tags}</span>
      </TermTooltip>
    ) : (
      m.zoomNames[z]
    );

  return (
    <section ref={box} className={styles.history} aria-label={m.historyArea}>
      <header className={styles.head}>
        <h2 className={styles.title}>
          <Term id="log" focusable={false} />
        </h2>
        <div className={styles.scope}>
          {(["all", "current"] as const).map((s) => (
            <TermTooltip key={s} id="branch">
              <button
                type="button"
                className={styles.scopeButton}
                aria-pressed={scope === s}
                onClick={() => setScope(s)}
              >
                {s === "all" ? m.scopeAll : m.scopeCurrent}
              </button>
            </TermTooltip>
          ))}
        </div>
        <span className={styles.spacer} />
        {first?.head && (
          <button type="button" className={styles.here} onClick={goHere}>
            {m.hereNow}
            <span className={styles.headName}>
              {" · "}
              <bdi>{first.currentBranch ?? first.head.slice(0, 7)}</bdi>
            </span>
          </button>
        )}
        <div className={styles.zoom}>
          <button
            type="button"
            className={styles.zoomButton}
            aria-label={m.zoomOut}
            disabled={zoom === "tags"}
            onClick={() => changeZoom("out")}
          >
            −
          </button>
          <span className={styles.zoomName}>{zoomName(zoom)}</span>
          <button
            type="button"
            className={styles.zoomButton}
            aria-label={m.zoomIn}
            disabled={zoom === "all"}
            onClick={() => changeZoom("in")}
          >
            +
          </button>
        </div>
        <span className={styles.status} role="status">
          {m.zoomStatus(m.zoomNames[zoom], shown.length)}
        </span>
      </header>
      {log.isPending && <EmptyState>{m.loading}</EmptyState>}
      {log.error && <ErrorNotice error={log.error} />}
      {pages &&
        (received === 0 ? (
          <EmptyState>{m.noCommits}</EmptyState>
        ) : (
          <RouteList
            nodes={shown}
            scene={scene}
            palette={palette}
            selectedId={selectedId}
            onSelect={select}
            onActivate={onActivate}
            onNearEnd={more}
            onWindow={onWindow}
            onUserScroll={() => setMenuOpen(false)}
            onListElement={(el) => {
              listEl.current = el;
            }}
            now={now}
            label={m.historyArea}
          >
            <StationMenu
              open={menuOpen && info !== null}
              onOpenChange={setMenuOpen}
              info={info}
              env={env}
              ctx={ctx}
              anchor={anchor}
              returnFocus={() => listEl.current?.focus()}
              isInside={(node) => listEl.current?.contains(node) ?? false}
            />
          </RouteList>
        ))}
      {pages && (first?.tipsCut ?? 0) > 0 && <p className={styles.note}>{m.tipsCut(first!.tipsCut)}</p>}
      {pages && !hasNextPage && received >= HISTORY_MAX && <p className={styles.note}>{m.moreCommits(received)}</p>}
    </section>
  );
}
