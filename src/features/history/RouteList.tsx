import {
  useCallback,
  useEffect,
  useRef,
  useState,
  type KeyboardEvent,
  type PointerEvent,
  type ReactNode,
} from "react";
import { formatDate } from "../../lib/time";
import { useMessages } from "../../messages";
import { useUiStore } from "../../state/uiStore";
import { rowAt } from "./layout/hit";
import { branchKey } from "./layout/lanes";
import { ROW_H } from "./layout/route";
import type { Scene } from "./layout/scene";
import type { RouteNode } from "./model";
import type { Palette } from "./render/Renderer";
import { SvgRenderer } from "./render/SvgRenderer";
import styles from "./RouteList.module.css";
import { scrollToRow, visibleRows } from "./useWindow";

/** 저장소에서 온 글자를 읽기 이름에 넣을 때의 길이. */
const MAX_SPOKEN = 200;
const spoken = (text: string) => (text.length > MAX_SPOKEN ? `${text.slice(0, MAX_SPOKEN)}…` : text);

type Props = {
  nodes: readonly RouteNode[];
  scene: Scene;
  palette: Palette;
  selectedId: string | null;
  onSelect: (id: string) => void;
  /** Enter·Space·ContextMenu·Shift+F10이나 역을 눌렀을 때. 접힌 구간이면 펼친다. */
  onActivate: (id: string) => void;
  /** 끝에서 두 화면 안으로 스크롤하면(다음 쪽 받기). */
  onNearEnd?: () => void;
  /** 그리는 행 범위가 바뀌면(줄 수 늦게 받기). */
  onWindow?: (first: number, last: number) => void;
  /** 사람이 스크롤하면(열린 역 메뉴를 닫는다). */
  onUserScroll?: () => void;
  /** 목록 요소를 알려 준다(역 메뉴가 포커스를 돌려주고 바깥 누름을 가린다). */
  onListElement?: (el: HTMLDivElement | null) => void;
  /** 목록 안에 함께 스크롤되며 놓이는 것(역 메뉴의 자리, 목록 안 좌표). */
  children?: ReactNode;
  /** 상대 날짜의 기준 시각(ms). */
  now: number;
  /** 목록 이름("버전 기록"). */
  label: string;
};

/**
 * 버전 기록 노선도 목록(r-log.png, D-092). 행 높이가 고정된 가상 스크롤이라 1만 행이어도 DOM 행은
 * 보이는 것 ± 10행뿐이다. 포커스는 목록 하나가 받고 `aria-activedescendant`가 고른 행을 가리킨다.
 * 클릭 판정은 장면 좌표로 한다(D-017).
 */
export function RouteList({
  nodes,
  scene,
  palette,
  selectedId,
  onSelect,
  onActivate,
  onNearEnd,
  onWindow,
  onUserScroll,
  onListElement,
  children,
  now,
  label,
}: Props) {
  const m = useMessages();
  const locale = useUiStore((s) => s.locale);
  const box = useRef<HTMLDivElement | null>(null);
  // 코드가 옮긴 스크롤(고른 행 보이기)은 사람의 스크롤이 아니다.
  const programmatic = useRef(false);
  const [scrollTop, setScrollTop] = useState(0);
  const [viewHeight, setViewHeight] = useState(600);
  const frame = useRef<number | null>(null);
  const total = nodes.length;
  const selected = selectedId === null ? -1 : nodes.findIndex((n) => n.id === selectedId);

  useEffect(() => {
    const el = box.current;
    if (!el || typeof ResizeObserver === "undefined") return;
    const observer = new ResizeObserver(([entry]) => {
      if (entry && entry.contentRect.height > 0) setViewHeight(entry.contentRect.height);
    });
    observer.observe(el);
    return () => observer.disconnect();
  }, []);
  useEffect(() => () => {
    if (frame.current !== null) cancelAnimationFrame(frame.current);
  }, []);

  const onScroll = () => {
    if (programmatic.current) programmatic.current = false;
    else onUserScroll?.();
    if (frame.current !== null) return;
    frame.current = requestAnimationFrame(() => {
      frame.current = null;
      const el = box.current;
      if (!el) return;
      setScrollTop(el.scrollTop);
      if (onNearEnd && el.scrollTop + viewHeight >= total * ROW_H - 2 * viewHeight) onNearEnd();
    });
  };

  const moveTo = useCallback(
    (row: number) => {
      const next = Math.max(0, Math.min(total - 1, row));
      const node = nodes[next];
      if (!node) return;
      onSelect(node.id);
      const el = box.current;
      if (el) {
        const top = scrollToRow(next, el.scrollTop, viewHeight, ROW_H);
        if (top !== el.scrollTop) {
          programmatic.current = true;
          el.scrollTop = top;
          setScrollTop(top);
        }
      }
    },
    [nodes, total, onSelect, viewHeight],
  );

  const onKeyDown = (e: KeyboardEvent<HTMLDivElement>) => {
    if (e.nativeEvent.isComposing || e.ctrlKey || e.altKey || e.metaKey) return;
    const page = Math.max(1, Math.floor(viewHeight / ROW_H) - 1);
    const at = selected < 0 ? -1 : selected;
    const keys: Record<string, () => void> = {
      ArrowDown: () => moveTo(at + 1),
      ArrowUp: () => moveTo(at < 0 ? 0 : at - 1),
      PageDown: () => moveTo(at + page),
      PageUp: () => moveTo(at - page),
      Home: () => moveTo(0),
      End: () => moveTo(total - 1),
    };
    const open = e.key === "Enter" || e.key === " " || e.key === "ContextMenu" || (e.key === "F10" && e.shiftKey);
    if (open) {
      e.preventDefault();
      if (selectedId !== null) onActivate(selectedId);
      return;
    }
    if (e.shiftKey) return;
    const run = keys[e.key];
    if (!run) return;
    e.preventDefault();
    run();
  };

  const onPointerDown = (e: PointerEvent<HTMLDivElement>) => {
    const el = box.current;
    if (!el || e.button !== 0) return;
    const rect = el.getBoundingClientRect();
    // 스크롤 막대를 누른 것은 고르지 않는다.
    if (e.clientX - rect.left > el.clientWidth) return;
    const row = rowAt(e.clientY - rect.top + el.scrollTop, ROW_H, total);
    const node = row === null ? undefined : nodes[row];
    if (!node) return;
    onSelect(node.id);
    onActivate(node.id);
  };


  const { first, last } = visibleRows(scrollTop, viewHeight, ROW_H, total);
  useEffect(() => {
    onWindow?.(first, last);
  }, [first, last, onWindow]);
  // 밖에서 고른 역("지금 여기", 확대·축소 뒤)이 화면 밖이면 보이게 스크롤한다.
  useEffect(() => {
    const el = box.current;
    if (!el || selected < 0) return;
    const top = scrollToRow(selected, el.scrollTop, viewHeight, ROW_H);
    if (top !== el.scrollTop) {
      programmatic.current = true;
      el.scrollTop = top;
      setScrollTop(top);
    }
  }, [selected, viewHeight]);
  const shown: number[] = [];
  for (let i = first; i < last; i++) shown.push(i);
  // 고른 행은 창 밖이어도 그려 aria-activedescendant가 있는 요소를 가리키게 한다.
  if (selected >= 0 && (selected < first || selected >= last)) shown.push(selected);

  const describe = (node: RouteNode, index: number): string => {
    if (node.kind === "fold") return m.routeFoldLabel(node.count);
    return m.routeRowLabel({
      summary: spoken(node.summary),
      author: spoken(node.author),
      date: formatDate(node.time, now, locale),
      branches: node.branches.map(spoken),
      tags: node.tags.map(spoken),
      head: node.isHead,
      merge: node.parents.length > 1,
      unpushed: node.unpushed,
      position: index + 1,
    });
  };

  return (
    <div
      ref={(el) => {
        box.current = el;
        onListElement?.(el);
      }}
      className={styles.list}
      role="listbox"
      aria-label={label}
      tabIndex={0}
      aria-activedescendant={selected >= 0 ? `route-row-${selected}` : undefined}
      onKeyDown={onKeyDown}
      onPointerDown={onPointerDown}
      onScroll={onScroll}
    >
      <div className={styles.content} style={{ height: total * ROW_H }}>
        <div className={styles.graph} style={{ width: scene.width }}>
          <SvgRenderer
            scene={scene}
            palette={palette}
            viewport={{ top: first * ROW_H, bottom: last * ROW_H }}
          />
        </div>
        {shown.map((i) => {
          const node = nodes[i]!;
          return (
            <div
              key={node.id}
              id={`route-row-${i}`}
              role="option"
              aria-selected={i === selected}
              aria-posinset={i + 1}
              aria-setsize={total}
              aria-label={describe(node, i)}
              className={`${styles.row} ${i === selected ? styles.selected : ""}`}
              style={{ top: i * ROW_H, height: ROW_H, paddingLeft: scene.width }}
            >
              {node.kind === "fold" ? (
                <span className={styles.fold}>{m.routeFold(node.count)}</span>
              ) : (
                <>
                  <span className={styles.name}>
                    <bdi className={styles.summary}>{node.summary}</bdi>
                    {node.unpushed && <span className={styles.mark}> · {m.notPushedMark}</span>}
                    {node.tags.map((t) => (
                      <span key={`t:${t}`} className={styles.mark}>
                        {" · "}
                        <bdi>{t}</bdi>
                      </span>
                    ))}
                    {node.branches.map((b) => (
                      <span key={`b:${b}`} className={styles.mark}>
                        {" · "}
                        <span className={styles.dot} style={{ background: palette(branchKey(b)) }} aria-hidden />
                        <bdi>{b}</bdi>
                      </span>
                    ))}
                  </span>
                  <bdi className={styles.author}>{node.author}</bdi>
                  <span className={styles.date}>{formatDate(node.time, now, locale)}</span>
                </>
              )}
            </div>
          );
        })}
        {children}
      </div>
    </div>
  );
}
