// 화면에서 프로젝트를 여는 유일한 훅(D-032). 앱을 켤 때 열기, 최근 목록, 폴더 고르기,
// 프로젝트 메뉴가 모두 이 훅 → project_open → Core::open_project를 거친다.
// 저장소 신뢰 확인도 그 한 곳(core)에 들어간다.
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { api, type OpenHow, type ProjectOpenResult, type TrustChoice } from "../../lib/api";
import { queryKeys } from "../../lib/queries";
import { useRepoStore } from "../../state/repoStore";
import { useTrustStore } from "./trustStore";

export function useOpenProject() {
  const openRepo = useRepoStore((s) => s.openRepo);
  const queryClient = useQueryClient();
  const ask = useTrustStore((s) => s.ask);
  const onResult = (result: ProjectOpenResult) => {
    if (result.kind === "opened") openRepo(result.repo);
    // 실행되는 설정이 있으면 신뢰 확인 창을 띄운다(D-039).
    if (result.kind === "needsTrust") ask({ folder: result.folder, keys: result.keys });
    void queryClient.invalidateQueries({ queryKey: queryKeys.recent });
  };
  const mutation = useMutation({
    mutationFn: ({ path, how }: { path: string; how: OpenHow }) => api.openProject(path, how),
    onSuccess: onResult,
  });
  const trustMutation = useMutation({
    mutationFn: ({ path, choice }: { path: string; choice: TrustChoice }) => api.trustProject(path, choice),
    onSuccess: onResult,
  });
  return {
    /** 열고 결과를 돌려준다. 열렸으면 작업 공간으로 바뀐다. */
    open: (path: string, how: OpenHow): Promise<ProjectOpenResult> => mutation.mutateAsync({ path, how }),
    /** 신뢰 확인에서 고른 결과로 연다. */
    trust: (path: string, choice: TrustChoice): Promise<ProjectOpenResult> =>
      trustMutation.mutateAsync({ path, choice }),
    isPending: mutation.isPending,
    error: mutation.error,
    reset: mutation.reset,
  };
}
