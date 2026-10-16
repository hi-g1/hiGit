// 첫 화면을 그리기 전에 설정과 OS 값을 읽어 루트 글자 크기를 정한다(D-149). 각각 0.8초까지만 기다리고,
// 늦거나 실패하면 기본값으로 그린 뒤 나중에 온 값을 useDisplaySync가 적용한다.
import type { QueryClient } from "@tanstack/react-query";
import { api, type AppSettingsView, type DisplayHintsView } from "./api";
import { applyDisplay } from "./display";
import { queryKeys } from "./queries";

export const PRELOAD_LIMIT_MS = 800;

function within<T>(read: () => Promise<T>, ms: number): Promise<T | undefined> {
  return new Promise((resolve) => {
    const timer = setTimeout(() => resolve(undefined), ms);
    let promise: Promise<T>;
    try {
      promise = read();
    } catch {
      clearTimeout(timer);
      resolve(undefined);
      return;
    }
    promise.then(
      (value) => {
        clearTimeout(timer);
        resolve(value);
      },
      () => {
        clearTimeout(timer);
        resolve(undefined);
      },
    );
  });
}

export async function preloadDisplay(client: QueryClient, ms = PRELOAD_LIMIT_MS): Promise<void> {
  const [settings, hints] = await Promise.all([
    within<AppSettingsView>(() => api.settings(), ms),
    within<DisplayHintsView>(() => api.displayHints(), ms),
  ]);
  if (settings) client.setQueryData(queryKeys.settings, settings);
  if (hints) client.setQueryData(queryKeys.displayHints, hints);
  applyDisplay(settings, hints);
}
