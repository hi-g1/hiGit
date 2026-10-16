import type { StationDialog } from "../ask";
import type { MenuCtx, MenuEnv } from "../items";

type Props = {
  dialog: StationDialog | null;
  onClose: () => void;
  env: MenuEnv;
  ctx: MenuCtx;
};

/** 역 메뉴가 연 대화상자를 그린다(태그·새 브랜치·고치기·파일 보기는 계획 008 E23~E25에서 채운다). */
export function StationDialogs({ dialog }: Props) {
  if (dialog === null) return null;
  return null;
}
