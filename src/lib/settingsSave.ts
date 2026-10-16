// 앱 설정 바꾸기(D-148). 화면에는 바로 반영하고(쿼리 캐시), 저장은 300ms 동안 모은 마지막 값만 보낸다.
// 단축키를 연타해도 저장 순서가 뒤바뀌지 않는다.
import type { QueryClient } from "@tanstack/react-query";
import { api, type AppSettingsPatch, type AppSettingsView } from "./api";
import { DEFAULT_SETTINGS } from "./display";
import { queryKeys } from "./queries";

export const SAVE_DELAY_MS = 300;

let pending: AppSettingsPatch = {};
let timer: ReturnType<typeof setTimeout> | null = null;

/** 지금 화면이 쓰는 설정. 읽지 못했으면 기본값. */
export function currentSettings(client: QueryClient): AppSettingsView {
  return client.getQueryData<AppSettingsView>(queryKeys.settings) ?? DEFAULT_SETTINGS;
}

function merge(base: AppSettingsView, patch: AppSettingsPatch): AppSettingsView {
  const { tipSeen, resetTips, ...plain } = patch;
  const next = { ...base };
  for (const [key, value] of Object.entries(plain) as [keyof AppSettingsView, unknown][]) {
    if (value !== undefined && value !== null) (next as Record<string, unknown>)[key] = value;
  }
  // 안내 비트는 칸 이름이 달라 따로 맞춘다(D-205).
  if (resetTips) next.tipsSeen = { firstBranch: false, firstConflict: false, firstSync: false, firstUndo: false };
  if (tipSeen) next.tipsSeen = { ...next.tipsSeen, [tipSeen]: true };
  return next;
}

export function changeSettings(client: QueryClient, patch: AppSettingsPatch): void {
  // 읽는 중인 옛 값이 바꾼 값을 덮지 않게 한다.
  void client.cancelQueries({ queryKey: queryKeys.settings });
  client.setQueryData<AppSettingsView>(queryKeys.settings, merge(currentSettings(client), patch));
  // 안내 닫기·다시 보기는 드물고 서로 합칠 수 없어(닫은 안내 둘) 기다리지 않고 따로 보낸다.
  if (patch.tipSeen || patch.resetTips) {
    void api.setSettings(patch).catch(() => undefined);
    return;
  }
  pending = { ...pending, ...patch };
  if (timer !== null) clearTimeout(timer);
  timer = setTimeout(() => {
    const send = pending;
    pending = {};
    timer = null;
    api.setSettings(send).then(
      (saved) => {
        // 그 사이에 또 바꿨으면 화면 값이 더 새롭다.
        if (timer === null) client.setQueryData(queryKeys.settings, saved);
      },
      // 저장하지 못해도 이번 실행 동안은 화면 값을 쓴다(core도 메모리로 내려간다).
      () => undefined,
    );
  }, SAVE_DELAY_MS);
}
