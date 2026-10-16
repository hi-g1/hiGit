import { useEffect, useRef, useState, type KeyboardEvent, type PointerEvent } from "react";
import { clampPane, PANE_MIN_REM, PANE_STEP_REM, paneKey, paneMaxRem, remPx } from "../lib/paneWidth";
import styles from "./Splitter.module.css";

type Props = {
  /** 읽기 이름. */
  label: string;
  /** 저장된 왼쪽 칸 폭(rem). 화면에 쓰이는 폭은 범위 안으로 맞춘 값이다. */
  valueRem: number;
  defaultRem: number;
  /** 끄는 동안·키·더블클릭마다. */
  onChange: (rem: number) => void;
  /** 끝났을 때 한 번(저장). */
  onCommit: (rem: number) => void;
  className?: string;
};

const round = (rem: number) => Math.round(rem * 10) / 10;

/**
 * 칸 나눔선(D-164). 왼쪽 칸과 오른쪽 칸 사이 1px 선이고 잡는 영역은 양쪽 4px씩 넓다. 끌거나 ←·→(1rem)로
 * 왼쪽 칸 폭을 바꾸고 더블클릭하면 기본 폭. 범위는 15rem~부모(칸들) 폭의 50%이고, 창이 줄어 범위를 넘으면
 * 부모의 CSS(`clamp`)가 맞춘다.
 */
export function Splitter({ label, valueRem, defaultRem, onChange, onCommit, className }: Props) {
  const self = useRef<HTMLDivElement>(null);
  const drag = useRef<{ x: number; rem: number; last: number } | null>(null);
  const [containerPx, setContainerPx] = useState(0);

  useEffect(() => {
    const parent = self.current?.parentElement;
    if (!parent) return;
    setContainerPx(parent.getBoundingClientRect().width);
    if (typeof ResizeObserver === "undefined") return;
    const observer = new ResizeObserver(([entry]) => {
      if (entry) setContainerPx(entry.contentRect.width);
    });
    observer.observe(parent);
    return () => observer.disconnect();
  }, []);

  const max = Math.max(PANE_MIN_REM, paneMaxRem(containerPx, remPx()));
  const shown = clampPane(valueRem, PANE_MIN_REM, max);

  /** 누를 때마다 부모 폭을 새로 잰다. */
  const range = () => {
    const width = self.current?.parentElement?.getBoundingClientRect().width || containerPx;
    const top = Math.max(PANE_MIN_REM, paneMaxRem(width, remPx()));
    return { top, now: clampPane(valueRem, PANE_MIN_REM, top) };
  };
  const set = (rem: number) => {
    onChange(rem);
    onCommit(rem);
  };

  const onPointerDown = (e: PointerEvent<HTMLDivElement>) => {
    if (e.button !== 0) return;
    e.preventDefault();
    e.currentTarget.setPointerCapture(e.pointerId);
    const { now } = range();
    drag.current = { x: e.clientX, rem: now, last: now };
    // 끄는 동안 글자가 선택되지 않게(base.css).
    document.documentElement.dataset.resizing = "";
  };
  const onPointerMove = (e: PointerEvent<HTMLDivElement>) => {
    const d = drag.current;
    if (!d) return;
    d.last = clampPane(d.rem + (e.clientX - d.x) / remPx(), PANE_MIN_REM, range().top);
    onChange(d.last);
  };
  const end = (e: PointerEvent<HTMLDivElement>) => {
    const d = drag.current;
    if (!d) return;
    drag.current = null;
    delete document.documentElement.dataset.resizing;
    if (e.currentTarget.hasPointerCapture(e.pointerId)) e.currentTarget.releasePointerCapture(e.pointerId);
    onCommit(d.last);
  };
  const onKeyDown = (e: KeyboardEvent<HTMLDivElement>) => {
    const dir = paneKey({ ...e, isComposing: e.nativeEvent.isComposing });
    if (!dir) return;
    e.preventDefault();
    const { top, now } = range();
    set(clampPane(now + dir * PANE_STEP_REM, PANE_MIN_REM, top));
  };

  return (
    <div
      ref={self}
      role="separator"
      aria-orientation="vertical"
      aria-label={label}
      aria-valuemin={PANE_MIN_REM}
      aria-valuemax={Number.isFinite(max) ? round(max) : undefined}
      aria-valuenow={round(shown)}
      tabIndex={0}
      className={`${styles.splitter} ${className ?? ""}`}
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onPointerUp={end}
      onPointerCancel={end}
      onDoubleClick={() => set(clampPane(defaultRem, PANE_MIN_REM, range().top))}
      onKeyDown={onKeyDown}
    />
  );
}
