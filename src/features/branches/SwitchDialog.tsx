import { useEffect, useRef, type KeyboardEvent } from "react";
import type { DirtyChoice, SwitchPlanView } from "../../lib/api";
import { useSwitchPlan } from "../../lib/queries";
import { useMessages, type Messages } from "../../messages";
import { Button } from "../../ui/Button";
import { DialogFrame, composing } from "../../ui/DialogFrame";
import { TermTooltip, useTermLabel } from "../../ui/Term";
import styles from "./SwitchDialog.module.css";
import type { SwitchTarget } from "./useSwitch";

type Props = {
  repo: string;
  target: SwitchTarget;
  /** 지금 브랜치(두고 갈 곳). */
  from: string;
  /** 상태에서 센 고친 파일 수. 조회가 오기 전 제목에 쓴다. */
  files: number;
  /** 실행 중인 선택. */
  pending: DirtyChoice | null;
  onChoose: (dirty: DirtyChoice) => void;
  onClose: () => void;
};

/** 새 브랜치로 옮길 수 없는 까닭. 조회가 실패하면 core가 실행 때 다시 보므로 막지 않는다. */
function carryReason(m: Messages, plan: SwitchPlanView | undefined, branch: string): string | null {
  if (!plan || plan.canCarry) return null;
  switch (plan.reason) {
    case "parkedThere":
      return m.switchDialog.parkedThere(branch);
    case "tooBig":
      return m.switchDialog.tooBig;
    default:
      return m.switchDialog.overlap(plan.blocked[0] ?? "", Math.max(plan.blockedTotal - 1, 0));
  }
}

/**
 * 고친 파일이 있는 채로 브랜치를 고르면 묻는다(D-165, D-195). 기본은 새 브랜치로 옮기기, 옮길 수 없으면
 * 까닭을 보이고 포커스는 여기 두기. 고른 것은 기억하지 않는다.
 */
export function SwitchDialog({ repo, target, from, files, pending, onChoose, onClose }: Props) {
  const m = useMessages();
  const leaveLabel = useTermLabel("park");
  const plan = useSwitchPlan(repo, target.branch, target.remote, true);
  const carryRef = useRef<HTMLButtonElement>(null);
  const leaveRef = useRef<HTMLButtonElement>(null);
  const data = plan.data;
  const loading = plan.isPending;
  const reason = carryReason(m, data, target.branch);

  // 조회가 오면 쓸 수 있는 쪽에 포커스. 그사이 고친 파일이 없어졌으면 묻지 않고 바꾼다.
  useEffect(() => {
    if (!data) return;
    if (data.dirtyFiles === 0) {
      onChoose("park");
      return;
    }
    (data.canCarry ? carryRef : leaveRef).current?.focus();
    // eslint-disable-next-line react-hooks/exhaustive-deps -- 조회 결과가 올 때 한 번만
  }, [data]);
  useEffect(() => {
    if (plan.isError) carryRef.current?.focus();
  }, [plan.isError]);

  const ignoreComposing = (e: KeyboardEvent) => {
    if (e.key === "Enter" && composing(e)) e.preventDefault();
  };

  return (
    <DialogFrame title={m.switchDialog.title(data?.dirtyFiles || files)} onClose={onClose} initialFocus={carryRef}>
      <div className={styles.options} onKeyDown={ignoreComposing}>
        <div className={styles.option}>
          <Button
            ref={carryRef}
            variant="primary"
            busy={loading || pending === "carry"}
            disabled={pending !== null}
            disabledReason={reason}
            onClick={() => onChoose("carry")}
          >
            {m.switchDialog.carry}
          </Button>
          <p className={styles.hint}>{m.switchDialog.carryHint(target.branch)}</p>
        </div>
        <div className={styles.option}>
          <TermTooltip id="park">
            <Button
              ref={leaveRef}
              busy={loading || pending === "park"}
              disabled={pending !== null}
              onClick={() => onChoose("park")}
            >
              {leaveLabel}
            </Button>
          </TermTooltip>
          <p className={styles.hint}>{m.switchDialog.leaveHint(from)}</p>
        </div>
      </div>
      <div className={styles.actions}>
        <Button variant="text" onClick={onClose}>
          {m.close}
        </Button>
      </div>
    </DialogFrame>
  );
}
