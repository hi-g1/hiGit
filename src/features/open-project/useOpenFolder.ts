// 폴더 고르기 창 → 프로젝트 열기(useOpenProject). 시작 화면의 "폴더 열기"와 어디서나 Ctrl+O가 쓴다.
import { useMutation } from "@tanstack/react-query";
import { ApiError } from "../../lib/api";
import { pickFolder } from "../../lib/dialog";
import { useRepoStore } from "../../state/repoStore";
import { useOpenProject } from "./useOpenProject";

export function useOpenFolder() {
  const project = useOpenProject();
  const setPendingInit = useRepoStore((s) => s.setPendingInit);
  const closeRepo = useRepoStore((s) => s.closeRepo);
  const mutation = useMutation({
    mutationFn: async () => {
      const path = await pickFolder();
      if (path === null) return;
      const result = await project.open(path, "picked");
      if (result.kind === "notARepo") {
        // 작업 공간에서 골랐어도 시작 화면으로 가서 버전 관리 시작을 묻는다.
        closeRepo();
        setPendingInit(result.folder);
      } else {
        setPendingInit(null);
      }
      if (result.kind === "missing") throw new ApiError({ code: "folderNotFound", detail: path });
    },
  });
  return {
    pick: () => {
      // 고르기 창이 이미 열려 있으면 다시 열지 않는다.
      if (!mutation.isPending) mutation.mutate();
    },
    isPending: mutation.isPending,
    error: mutation.error,
  };
}
