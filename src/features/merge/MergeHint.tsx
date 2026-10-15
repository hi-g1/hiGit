import { useMergeHint } from "../../lib/queries";
import { useMessages } from "../../messages";
import { Button } from "../../ui/Button";
import { TermTooltip } from "../../ui/Term";
import { Tooltip } from "../../ui/Tooltip";
import styles from "./MergeHint.module.css";
import { useMergeAction } from "./useMergeAction";

/**
 * 노선도 옆 합치기 제안(D-048, r-mergehint). 기본 브랜치에 새 버전이 있을 때만 보인다.
 * 이름과 숫자만 쓴다(D-049). 이 화면의 주 버튼은 버전 저장이라 이 버튼은 테두리다.
 */
export function MergeHint({ repo, onDone }: { repo: string; onDone: (text: string) => void }) {
  const m = useMessages();
  const hint = useMergeHint(repo).data;
  const { run, pending } = useMergeAction(repo, onDone);
  if (!hint || hint.kind !== "bring" || hint.branch === null) return null;
  const branch = hint.branch;
  return (
    <div className={styles.hint}>
      <span className={styles.count}>{m.mainNewVersions(branch, hint.behind)}</span>
      <TermTooltip id="merge">
        <Button disabled={pending} onClick={() => void run("merge", branch)}>
          {m.bringMerge(branch)}
        </Button>
      </TermTooltip>
      {hint.overlapCount > 0 && (
        <Tooltip content={hint.overlap.join("\n")}>
          <span className={styles.overlap} tabIndex={0}>
            {m.overlapFiles(hint.overlapCount)}
          </span>
        </Tooltip>
      )}
    </div>
  );
}
