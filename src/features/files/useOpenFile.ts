// 고친 파일을 폴더에서 보기·기본 앱으로 열기(D-216). 경로는 저장소 기준으로만 보내고 검사는 core가 한다.
import { ApiError, api } from "../../lib/api";
import { useMessages } from "../../messages";
import { useToastStore } from "../../ui/toastStore";

export function useOpenFile(repo: string) {
  const m = useMessages();
  const showError = useToastStore((s) => s.showError);
  const reveal = (path: string) => void api.fileReveal(repo, path).catch((e: unknown) => showError(e));
  const open = (path: string) =>
    void api.fileOpen(repo, path).catch((e: unknown) => {
      // 실행될 수 있는 파일이라 막았으면 폴더에서 보기를 해결 버튼으로 준다.
      if (e instanceof ApiError && e.code === "openRefused") showError(e, { label: m.revealInFolder, onClick: () => reveal(path) });
      else showError(e);
    });
  return { reveal, open };
}
