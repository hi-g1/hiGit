import type { SceneStation } from "../layout/scene";
import type { Palette, Renderer, Viewport } from "./Renderer";
import styles from "./SvgRenderer.module.css";

const inView = (y: number, v: Viewport | undefined, pad: number) => !v || (y >= v.top - pad && y <= v.bottom + pad);

/**
 * 노선도를 SVG로 그린다. 선과 점만 브랜치 색이고, 글자는 --text·--text-muted다(D-031).
 * 역 이름은 저장소에서 온 글자라 React 글자로만 그린다(D-037).
 */
export const SvgRenderer: Renderer = ({ scene, palette, label, viewport }) => (
  <svg
    className={styles.svg}
    width={scene.width}
    height={scene.height}
    viewBox={`0 0 ${scene.width} ${scene.height}`}
    role={label ? "img" : undefined}
    aria-label={label}
    aria-hidden={label ? undefined : true}
  >
    {scene.lines
      .filter((l) => !viewport || !l.span || (l.span[1] >= viewport.top && l.span[0] <= viewport.bottom))
      .map((l, i) => (
        <path
          key={i}
          d={l.d}
          fill="none"
          className={styles.line}
          stroke={palette(l.color)}
          strokeWidth={l.width}
          strokeDasharray={l.dashed ? "2 6" : undefined}
          strokeLinecap="round"
        />
      ))}
    {scene.stations
      .filter((s) => inView(s.y, viewport, s.r + 12))
      .map((s, i) => (
        <Station key={s.id ?? i} station={s} palette={palette} />
      ))}
    {scene.tags.map((t, i) => (
      <g key={i} className={styles.tag}>
        <rect x={t.x - 32} y={t.y - 12} width={64} height={18} rx={3} />
        <text x={t.x} y={t.y + 1} textAnchor="middle">
          {t.text}
        </text>
      </g>
    ))}
    {scene.texts
      .filter((t) => inView(t.y, viewport, 24))
      .map((t, i) => (
        <text key={i} x={t.x} y={t.y} textAnchor={t.anchor} className={t.tone === "text" ? styles.text : styles.muted}>
          {t.text}
        </text>
      ))}
  </svg>
);

function Station({ station: s, palette }: { station: SceneStation; palette: Palette }) {
  const color = palette(s.color);
  const title = s.title ? <title>{s.title}</title> : null;
  if (s.kind === "fold") {
    // 접힌 구간: 작은 마름모.
    return (
      <path
        d={`M ${s.x} ${s.y - s.r} L ${s.x + s.r} ${s.y} L ${s.x} ${s.y + s.r} L ${s.x - s.r} ${s.y} Z`}
        className={styles.station}
        stroke={color}
        strokeWidth={s.stroke}
      >
        {title}
      </path>
    );
  }
  return (
    <g>
      {s.ring !== undefined && <circle cx={s.x} cy={s.y} r={s.ring} className={styles.ring} />}
      <circle
        cx={s.x}
        cy={s.y}
        r={s.r}
        className={styles.station}
        stroke={color}
        strokeWidth={s.stroke}
        strokeDasharray={s.kind === "open" ? "3 3" : undefined}
      >
        {title}
      </circle>
    </g>
  );
}
