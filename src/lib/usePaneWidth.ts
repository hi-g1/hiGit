import { useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { useAppSettings } from "./queries";
import { changeSettings } from "./settingsSave";

/** 칸 폭을 저장하는 설정 칸(화면마다 따로, D-164). */
export type PaneKey = "workspacePane" | "conflictPane" | "filesPane";

/**
 * 나눔선으로 바꾸는 왼쪽 칸 폭(rem). 끄는 동안은 화면 안에서만 바꾸고, 끝나면 앱 설정에 저장한다(앱 전체에 하나).
 */
export function usePaneWidth(key: PaneKey, defaultRem: number) {
  const client = useQueryClient();
  const saved = useAppSettings().data?.[key] ?? null;
  const [live, setLive] = useState<number | null>(null);
  const rem = live ?? saved ?? defaultRem;
  return {
    rem,
    /** `<Splitter>`에 그대로 넘기는 값. */
    splitter: {
      valueRem: rem,
      defaultRem,
      onChange: setLive,
      onCommit: (next: number) => {
        setLive(null);
        changeSettings(client, { [key]: next });
      },
    },
  };
}
