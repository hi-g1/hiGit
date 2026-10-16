import { useEffect } from "react";
import { applyDisplay } from "./display";
import { useAppSettings, useDisplayHints } from "./queries";

/** 설정이나 OS 값(창으로 돌아오면 다시 읽음)이 바뀌면 화면에 적용한다(D-146, D-147). */
export function useDisplaySync(): void {
  const settings = useAppSettings().data;
  const hints = useDisplayHints().data;
  useEffect(() => {
    applyDisplay(settings, hints);
  }, [settings, hints]);
}
