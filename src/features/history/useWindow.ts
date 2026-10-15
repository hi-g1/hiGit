// 고정 행 높이 가상 스크롤의 창 계산. DOM 없음.

/** 화면에 그릴 행 범위 [first, last). 위아래로 `overscan`행씩 더 그린다. */
export function visibleRows(
  scrollTop: number,
  viewHeight: number,
  rowHeight: number,
  rows: number,
  overscan = 10,
): { first: number; last: number } {
  const first = Math.max(0, Math.floor(scrollTop / rowHeight) - overscan);
  const last = Math.min(rows, Math.ceil((scrollTop + viewHeight) / rowHeight) + overscan);
  return { first, last: Math.max(first, last) };
}

/** 고른 행이 화면에 보이도록 할 때의 새 scrollTop. 이미 보이면 그대로. */
export function scrollToRow(row: number, scrollTop: number, viewHeight: number, rowHeight: number): number {
  const top = row * rowHeight;
  const bottom = top + rowHeight;
  if (top < scrollTop) return top;
  if (bottom > scrollTop + viewHeight) return bottom - viewHeight;
  return scrollTop;
}
