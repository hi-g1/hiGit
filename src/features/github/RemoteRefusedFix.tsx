import { api } from "../../lib/api";
import { useHostingContext } from "../../lib/queries";
import { useMessages } from "../../messages";
import { useRepoStore } from "../../state/repoStore";
import { Button } from "../../ui/Button";
import { useToastStore } from "../../ui/toastStore";

/**
 * 서버가 올리기를 거절한 오류 줄의 단추(D-183 ⑤). 열린 저장소가 GitHub면 브랜치 보호 규칙 설정 페이지를
 * 연다(주소는 core가 만든다). 바꿀 수 있는 사람은 저장소 관리자뿐이라 그 한 줄을 함께 보인다.
 */
export function RemoteRefusedFix() {
  const m = useMessages();
  const repo = useRepoStore((s) => s.repo?.path ?? null);
  const github = useHostingContext(repo ?? "", repo !== null).data?.provider === "github";
  const showError = useToastStore((s) => s.showError);
  if (!repo || !github) return null;
  return (
    <>
      <Button
        variant="secondary"
        onClick={() => void api.openHostingPage(repo, { kind: "branchSettings" }).catch(showError)}
      >
        {m.nameConfirm.viewOnGithub}
      </Button>
      <span>{m.nameConfirm.adminOnly}</span>
    </>
  );
}
