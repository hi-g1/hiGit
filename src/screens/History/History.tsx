import { useQueryClient } from "@tanstack/react-query";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { buildRoute, ROW_H, verticalScene } from "../../features/history/layout/route";
import { simplify, ZOOMS, type Zoom } from "../../features/history/layout/simplify";
import type { StationDialog } from "../../features/history/menu/ask";
import { StationDialogs } from "../../features/history/menu/dialogs/StationDialogs";
import { headAncestors, stationInfo } from "../../features/history/menu/info";
import { eligible, extend, headLine, rangeIds as rangeBetween } from "../../features/history/range";
import type { MenuCtx, MenuEnv } from "../../features/history/menu/items";
import { StationMenu } from "../../features/history/menu/StationMenu";
import { toRouteNodes } from "../../features/history/model";
import { branchPalette } from "../../features/history/render/palette";
import { RouteList } from "../../features/history/RouteList";
import { useCommitStats } from "../../features/history/stats";
import { HISTORY_MAX, useLogPages, useStatus } from "../../lib/queries";
import { useShortcut } from "../../lib/useShortcut";
import { useUiScale } from "../../lib/useUiScale";
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
export function History({
  repo,
  onDone,
  otherOperation = false,
}: {
  repo: string;
  onDone: (text: string) => void;
  /** 다른 도구가 시작한 작업이 멈춰 있다(편집 항목을 흐린다). */
  otherOperation?: boolean;
}) {
  const m = useMessages();
  const scale = useUiScale();
  const queryClient = useQueryClient();
  const show = useToastStore((s) => s.show);
  const status = useStatus(repo).data;
  const changedFiles = status?.files.length ?? 0;
  const stagedFiles = status?.files.filter((f) => f.staged).length ?? 0;
  const setView = useRepoStore((s) => s.setView);
  const { selectedId, anchorId, rangeIds, zoom, scope, expanded, select, selectRange, setZoom, setScope, expand } =
    useHistoryStore();
  const openRewrite = useRepoStore((s) => s.openRewrite);
  const log = useLogPages(repo, scope);
  const { hasNextPage, isFetchingNextPage, fetchNextPage } = log;
  const pages = log.data?.pages;
  const first = pages?.[0];
  const [now] = useState(() => Date.now());
  const box = useRef<HTMLDivElement>(null);

  useEffect(() => {
    // Esc: 열린 메뉴(Radix가 처리) → 고른 구간 지우기 → 홈.
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key !== "Escape" || e.defaultPrevented) return;
      if (useHistoryStore.getState().rangeIds.length > 0) {
        e.preventDefault();
        useHistoryStore.getState().clearRange();
        return;
      }
      setView("home");
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
    if (shown.length * ROW_H * scale < 2 * 600 && hasNextPage && !isFetchingNextPage && auto.current < AUTO_PAGES) {
      auto.current += 1;
      void fetchNextPage();
    }
  }, [shown.length, scale, hasNextPage, isFetchingNextPage, fetchNextPage]);

  // "지금 여기": 지금 위치가 아직 안 왔으면 이어 받아 찾는다(최대 25쪽).
  const seeking = useRef(0);
  const head = shown.find((n) => n.isHead);
  const goHere = () => {
    if (head) {
      select(head.id);
      return;
    }
    // 더 받을 쪽이 없으면 찾지 않는다(남은 표시가 나중에 선택을 옮기지 않게).
    if (!hasNextPage) return;
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
        // 펼친 구간의 가장 새 버전을 고른다.
        const newest = node.statOids[0];
        if (newest) select(newest);
      } else if (node?.kind === "commit") {
        setMenuOpen((open) => !(open && id === selectedId));
      }
    },
    [shown, expand, select, selectedId],
  );
  // 오른쪽 클릭: 눌린 역을 고르고 연다(이미 열려 있으면 그대로).
  const onContextMenu = useCallback(
    (id: string) => {
      const node = shown.find((n) => n.id === id);
      if (node?.kind === "fold") onActivate(id);
      else if (node?.kind === "commit") setMenuOpen(true);
    },
    [shown, onActivate],
  );
  const [dialog, setDialog] = useState<StationDialog | null>(null);
  const ancestors = useMemo(() => headAncestors(shown), [shown]);
  const line = useMemo(() => headLine(shown), [shown]);
  const rangeSet = useMemo(() => new Set(rangeIds), [rangeIds]);
  // 여러 역 선택(D-106): 지금 브랜치 선 위에서만, 곁가지 행은 건너뛴다.
  const onExtend = (dir: "next" | "prev") => {
    const anchor = anchorId ?? selectedId;
    if (anchor === null || selectedId === null || !line.includes(anchor) || !line.includes(selectedId)) return;
    setMenuOpen(false);
    const next = extend(line, selectedId, dir);
    const ids = rangeBetween(line, anchor, next);
    selectRange(anchor, next, ids.length > 1 ? ids : []);
  };
  const onExtendTo = (id: string) => {
    const anchor = anchorId ?? selectedId ?? id;
    const ids = rangeBetween(line, anchor, id);
    setMenuOpen(false);
    if (ids.length > 1) selectRange(anchor, id, ids);
    else select(id);
  };
  const canRewrite = eligible(rangeIds, shown, zoom);
  const selectedRow = selectedId === null ? -1 : shown.findIndex((n) => n.id === selectedId);
  const selectedNode = selectedRow >= 0 ? shown[selectedRow] : undefined;
  const info = selectedNode ? stationInfo(selectedNode, route.layout.onHeadLine, ancestors) : null;
  const anchor = {
    // 목록 안 좌표(px): 장면 단위에 배율을 곱한다(D-153).
    x: (scene.stations.find((s) => s.id === selectedId)?.x ?? 0) * scale + 16,
    y: (selectedRow * ROW_H + ROW_H / 2) * scale,
  };
  const env: MenuEnv = {
    repo,
    currentBranch: first?.currentBranch ?? null,
    changedFiles,
    stagedFiles,
    operation: otherOperation ? "other" : "none",
    upstream: status?.branch.upstream ?? null,
    defaultBranch: first?.defaultBranch ?? null,
    zoom,
    headLine: line,
  };
  const ctx: MenuCtx = {
    repo,
    toast: show,
    queryClient,
    onDone,
    ask: (d) => {
      setMenuOpen(false);
      setDialog(d);
    },
    openActivity: () => setView("activity"),
    openRewrite,
    headLine: () => line,
    m,
  };

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
        {canRewrite && (
          <button type="button" className={styles.here} onClick={() => openRewrite(rangeIds)}>
            {m.rangeRewrite(rangeIds.length)}
          </button>
        )}
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
        {/* 화면에 보이지 않고 스크린 리더만 읽는 상태 문장(용어는 t()로 문장 속에) */}
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
            onContextMenu={onContextMenu}
            rangeIds={rangeSet}
            onExtend={onExtend}
            onExtendTo={onExtendTo}
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
      <StationDialogs
        dialog={dialog}
        onClose={() => {
          setDialog(null);
          // 대화상자가 사라진 다음에 목록으로 돌아간다.
          setTimeout(() => listEl.current?.focus(), 0);
        }}
        env={env}
        ctx={ctx}
      />
      {pages && (first?.tipsCut ?? 0) > 0 && (
        <p className={styles.note}>
          <Term id="branch" /> {m.tipsCut(first!.tipsCut)}
        </p>
      )}
      {pages && !hasNextPage && received >= HISTORY_MAX && <p className={styles.note}>{m.moreCommits(received)}</p>}
    </section>
  );
}
