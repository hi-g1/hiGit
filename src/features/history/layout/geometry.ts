// 노선도 배치가 쓰는 글자 폭·경로 도우미. DOM 없이 계산한다(D-017: 판정·배치는 장면 좌표로).

/** 한글·한자·전각·이모지처럼 한 글자가 1em인지. */
function isWide(code: number): boolean {
  return (
    (code >= 0x1100 && code <= 0x11ff) ||
    (code >= 0x2e80 && code <= 0xa4cf) ||
    (code >= 0xac00 && code <= 0xd7a3) ||
    (code >= 0xf900 && code <= 0xfaff) ||
    (code >= 0xfe30 && code <= 0xfe4f) ||
    (code >= 0xff00 && code <= 0xff60) ||
    (code >= 0xffe0 && code <= 0xffe6) ||
    code > 0xffff
  );
}

/** 글자 폭 추정(px). 번들 글꼴(Pretendard) 기준으로 넓은 글자 1em, 그 밖 0.55em. */
export function textWidth(text: string, px: number): number {
  let em = 0;
  for (const ch of text) em += isWide(ch.codePointAt(0) ?? 0) ? 1 : 0.55;
  return em * px;
}

/** `maxPx` 안에 들어가게 뒤를 "…"로 줄인다. 들어가면 그대로. */
export function fitText(text: string, maxPx: number, px: number): string {
  if (textWidth(text, px) <= maxPx) return text;
  const chars = [...text];
  const ellipsis = textWidth("…", px);
  let width = 0;
  let out = "";
  for (const ch of chars) {
    const w = textWidth(ch, px);
    if (width + w + ellipsis > maxPx) break;
    width += w;
    out += ch;
  }
  return out ? `${out}…` : "…";
}

/** 가로로 가다 다른 높이로 부드럽게 옮겨 가는 곡선 조각(홈 노선도의 갈라짐). */
export function curveH(x1: number, y1: number, x2: number, y2: number): string {
  const mid = (x1 + x2) / 2;
  return `M ${x1} ${y1} C ${mid} ${y1}, ${mid} ${y2}, ${x2} ${y2}`;
}
