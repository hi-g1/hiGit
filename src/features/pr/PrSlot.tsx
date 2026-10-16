import { useGitHubStatus, useHostingContext, usePrForBranch } from "../../lib/queries";
import { useMessages } from "../../messages";
import { useRepoStore } from "../../state/repoStore";
import { Button } from "../../ui/Button";
import { TermTooltip } from "../../ui/Term";
import { useSettingsStore } from "../settings/settingsStore";
import { stateLabel } from "./model";
import { usePrDialogStore } from "./prDialogStore";

/**
 * 브랜치 목록의 "검토 요청" 자리(D-071, D-113). GitHub 프로젝트에서 합치기보다 먼저 권한다. 이 브랜치의
 * 요청이 있으면 번호·상태(누르면 검토 요청 화면에서 그 줄), 없으면 새 검토 요청, 연결 전이면 연결 패널.
 * 조회 중에는 같은 자리를 꺼진 단추로 지킨다(깜박이지 않게).
 */
export function PrSlot({ repo, onClose }: { repo: string; onClose: () => void }) {
  const m = useMessages();
  const status = useGitHubStatus().data;
  const context = useHostingContext(repo).data;
  const connected = status?.connected === true;
  const prs = usePrForBranch(repo, context?.head ?? null, connected);
  const openSettings = useSettingsStore((s) => s.openSettings);
  const openPrDialog = usePrDialogStore((s) => s.openPrDialog);
  const openPr = useRepoStore((s) => s.openPr);
  const mine = prs.data?.items.filter((r) => r.isCurrentBranch) ?? [];
  const existing = mine.find((r) => r.state === "open") ?? mine[0] ?? null;

  if (!connected) {
    return (
      <TermTooltip id="pr">
        <Button
          onClick={() => {
            onClose();
            openSettings(true);
          }}
        >
          {m.pr.request}
        </Button>
      </TermTooltip>
    );
  }
  if (existing) {
    return (
      <Button
        onClick={() => {
          onClose();
          openPr(existing.number);
        }}
      >
        {m.pr.slotExisting(existing.number, stateLabel(existing, m) ?? m.pr.draftLabel)}
      </Button>
    );
  }
  return (
    <TermTooltip id="pr">
      <Button
        disabled={context?.head != null && prs.isPending}
        onClick={() => {
          onClose();
          openPrDialog();
        }}
      >
        {m.pr.request}
      </Button>
    </TermTooltip>
  );
}
