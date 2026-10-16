import { useUiStore } from "../state/uiStore";

/** 화면 배율(루트 글자 크기 / 16px). 노선도처럼 px로 그리는 곳이 곱한다(D-153). */
export function useUiScale(): number {
  return useUiStore((s) => s.uiScale);
}
