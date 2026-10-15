import { MAP_HEIGHT } from "../layout/twoLine";
import type { Renderer } from "./Renderer";
import styles from "./SvgRenderer.module.css";

/**
 * 노선도를 SVG로 그린다. 선과 점만 브랜치 색이고, 글자는 --text·--text-muted다(D-031).
 * 역 이름은 저장소에서 온 글자라 React 글자로만 그린다(D-037).
 */
export const SvgRenderer: Renderer = ({ scene, colors, label }) => (
  <svg
    className={styles.svg}
    width={scene.width}
    height={MAP_HEIGHT}
    viewBox={`0 0 ${scene.width} ${MAP_HEIGHT}`}
    role="img"
    aria-label={label}
  >
    {scene.lines.map((l, i) => (
      <path
        key={i}
        d={l.d}
        fill="none"
        stroke={l.kind === "main" ? colors.main : colors.current}
        strokeWidth={l.kind === "main" ? 3 : 4}
        strokeDasharray={l.dashed ? "2 6" : undefined}
        strokeLinecap="round"
      />
    ))}
    {scene.stations.map((s, i) => (
      <circle
        key={i}
        cx={s.x}
        cy={s.y}
        r={s.open ? 9 : 6}
        className={styles.station}
        stroke={s.line === "main" ? colors.main : colors.current}
        strokeWidth={2.5}
        strokeDasharray={s.open ? "3 3" : undefined}
      >
        {s.label && <title>{s.label}</title>}
      </circle>
    ))}
    {scene.tag && (
      <g className={styles.tag}>
        <rect x={scene.tag.x - 32} y={scene.tag.y - 12} width={64} height={18} rx={3} />
        <text x={scene.tag.x} y={scene.tag.y + 1} textAnchor="middle">
          {scene.tag.text}
        </text>
      </g>
    )}
    {scene.texts.map((t, i) => (
      <text key={i} x={t.x} y={t.y} textAnchor={t.anchor} className={t.tone === "text" ? styles.text : styles.muted}>
        {t.text}
      </text>
    ))}
  </svg>
);
