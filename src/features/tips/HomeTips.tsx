import type { TipId } from "../../lib/api";
import { useMessages } from "../../messages";
import { useRepoStore } from "../../state/repoStore";
import { Button } from "../../ui/Button";
import { TipLine } from "../../ui/TipLine";
import { useTipStore } from "./tipStore";
import { useTip } from "./useTip";

/** 홈 알림 줄 아래의 처음 하는 일 안내(첫 브랜치·첫 동기화·첫 되돌리기, D-205). 그 일을 한 직후에만 뜬다. */
export function HomeTips() {
  const due = useTipStore((s) => s.due);
  return (
    <>
      {(["firstBranch", "firstSync", "firstUndo"] as const)
        .filter((id) => due.includes(id))
        .map((id) => (
          <HomeTip key={id} id={id} />
        ))}
    </>
  );
}

function HomeTip({ id }: { id: TipId }) {
  const m = useMessages();
  const tip = useTip(id);
  const setView = useRepoStore((s) => s.setView);
  if (!tip.visible) return null;
  const action =
    id === "firstUndo" ? (
      <Button variant="text" onClick={() => setView("activity")}>
        {m.openActivity}
      </Button>
    ) : undefined;
  return (
    <TipLine onDismiss={tip.dismiss} action={action}>
      {m.tip[id]}
    </TipLine>
  );
}
