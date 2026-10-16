import { useQueryClient } from "@tanstack/react-query";
import { useEffect } from "react";
import { api } from "../../lib/api";
import { queryKeys } from "../../lib/queries";
import { useMessages } from "../../messages";

/**
 * 앱을 켤 때와 언어를 바꿨을 때 오른쪽 클릭 메뉴를 맞춘다(D-214). 설치한 앱은 처음 켤 때 등록되고, 켠 메뉴는 앱을
 * 옮기거나 언어를 바꾸면 다시 등록된다. 실패는 조용히 넘긴다(core가 로그에 남긴다).
 */
export function useShellMenuSync(enabled: boolean): void {
  const client = useQueryClient();
  const label = useMessages().shellMenuLabel;
  useEffect(() => {
    if (!enabled) return;
    let stale = false;
    api
      .shellMenuSync(label)
      .then((view) => {
        if (!stale) client.setQueryData(queryKeys.shellMenu(label), view);
      })
      .catch(() => undefined);
    return () => {
      stale = true;
    };
  }, [enabled, label, client]);
}
