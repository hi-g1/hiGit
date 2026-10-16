import { useState, type CSSProperties } from "react";
import type { ImageBlob } from "../../lib/api";
import { useMessages } from "../../messages";
import { Choice } from "../../ui/Choice";
import styles from "./ImageCompare.module.css";

type Mode = "side" | "overlay" | "swipe";
type Size = { w: number; h: number } | null;

/**
 * 이미지 비교(D-206): 나란히·겹쳐 보기·밀어 보기. 두 쪽은 `data:` 주소의 `<img>`로만 그린다(svg도, D-037).
 * 한쪽만 있으면 그 한 장과 "새 파일"/"삭제". 큰 이미지는 칸 폭에 맞춰 줄이고 "원본 크기"로 그대로 본다.
 */
export function ImageCompare({ name, before, after }: { name: string; before: ImageBlob | null; after: ImageBlob | null }) {
  const m = useMessages();
  const [mode, setMode] = useState<Mode>("side");
  const [opacity, setOpacity] = useState(50);
  const [swipe, setSwipe] = useState(50);
  const [original, setOriginal] = useState(false);
  const [sizes, setSizes] = useState<{ before: Size; after: Size }>({ before: null, after: null });
  const src = (b: ImageBlob) => `data:${b.mime};base64,${b.data}`;
  const loaded = (side: "before" | "after") => (e: React.SyntheticEvent<HTMLImageElement>) => {
    const img = e.currentTarget;
    setSizes((s) => ({ ...s, [side]: { w: img.naturalWidth, h: img.naturalHeight } }));
  };
  const dim = (s: Size) => (s ? `${s.w}×${s.h}` : "");
  const image = (b: ImageBlob, side: "before" | "after", className = "") => (
    <img
      className={`${styles.img} ${className}`}
      src={src(b)}
      alt={`${name} · ${side === "before" ? m.imageCompare.oldSide : m.imageCompare.newSide}`}
      onLoad={loaded(side)}
      draggable={false}
    />
  );
  const frame = (child: React.ReactNode) => (
    <div className={`${styles.checker} ${original ? styles.original : ""}`}>{child}</div>
  );
  const originalToggle = (
    <button type="button" className={styles.size} aria-pressed={original} onClick={() => setOriginal((o) => !o)}>
      {m.imageCompare.original}
    </button>
  );

  if (!before || !after) {
    const only = (after ?? before)!;
    const side = after ? "after" : "before";
    return (
      <div className={styles.compare}>
        <div className={styles.bar}>
          <span className={styles.size}>
            {after ? m.changeKind.untracked : m.changeKind.deleted} · {dim(sizes[side])}
          </span>
          <span className={styles.spacer} />
          {originalToggle}
        </div>
        {frame(image(only, side))}
      </div>
    );
  }

  return (
    <div className={styles.compare}>
      <div className={styles.bar}>
        <Choice
          label={m.imageCompare.modes}
          value={mode}
          options={[
            { value: "side", label: m.imageCompare.sideBySide },
            { value: "overlay", label: m.imageCompare.overlay },
            { value: "swipe", label: m.imageCompare.swipe },
          ]}
          onChange={setMode}
        />
        {mode === "overlay" && (
          <input
            className={styles.range}
            type="range"
            min={0}
            max={100}
            value={opacity}
            aria-label={m.imageCompare.opacity}
            onChange={(e) => setOpacity(Number(e.target.value))}
          />
        )}
        {mode === "swipe" && (
          <input
            className={styles.range}
            type="range"
            min={0}
            max={100}
            value={swipe}
            aria-label={m.imageCompare.position}
            onChange={(e) => setSwipe(Number(e.target.value))}
          />
        )}
        <span className={styles.size}>
          {dim(sizes.before)} → {dim(sizes.after)}
        </span>
        <span className={styles.spacer} />
        {originalToggle}
      </div>
      {mode === "side" ? (
        <div className={styles.side}>
          <figure className={styles.figure}>
            <figcaption className={styles.caption}>{m.imageCompare.oldSide}</figcaption>
            {frame(image(before, "before"))}
          </figure>
          <figure className={styles.figure}>
            <figcaption className={styles.caption}>{m.imageCompare.newSide}</figcaption>
            {frame(image(after, "after"))}
          </figure>
        </div>
      ) : (
        frame(
          <div
            className={styles.stack}
            style={{ "--top-opacity": opacity / 100, "--swipe": `${swipe}%` } as CSSProperties}
          >
            {image(before, "before")}
            {image(after, "after", mode === "overlay" ? styles.top : styles.clip)}
          </div>,
        )
      )}
    </div>
  );
}
