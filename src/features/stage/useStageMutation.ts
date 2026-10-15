import { useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "../../lib/api";
import { queryKeys } from "../../lib/queries";

/** 이번 버전에 넣기·빼기. 성공하면 변경 사항을 다시 읽는다(감시 이벤트를 기다리지 않음). */
export function useStageMutation(folder: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ include, paths }: { include: boolean; paths: string[] }) =>
      api.executeAction(
        folder,
        include ? { id: "stage", input: { paths } } : { id: "unstage", input: { paths } },
      ),
    onSettled: () => queryClient.invalidateQueries({ queryKey: queryKeys.status(folder) }),
  });
}
