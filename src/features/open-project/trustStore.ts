import { create } from "zustand";
import type { TrustRedirect } from "../../lib/api";

/**
 * config: 실행되는 설정이나 올리는 곳을 바꾸는 설정(`redirects`, D-212)이 있음. owner: 폴더 주인이 다른
 * 사용자(safe.directory). canDisable이 거짓이면 덮어써서 끌 수 없는 설정이 있어 "끄고 열기"를 보이지 않는다.
 */
export type PendingTrust = {
  kind: "config" | "owner";
  folder: string;
  keys: string[];
  redirects?: TrustRedirect[];
  canDisable: boolean;
};

type TrustState = {
  /** 믿을지 물어야 하는 프로젝트(D-039). 있으면 신뢰 확인 창이 뜬다. */
  pending: PendingTrust | null;
  ask: (p: PendingTrust) => void;
  clear: () => void;
};

export const useTrustStore = create<TrustState>((set) => ({
  pending: null,
  ask: (pending) => set({ pending }),
  clear: () => set({ pending: null }),
}));
