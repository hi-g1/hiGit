import { DEFAULT_SETTINGS } from "../../lib/display";
import { useAppSettings, useForcePushPlan } from "../../lib/queries";
import { useMessages } from "../../messages";
import { Button } from "../../ui/Button";
import { Term } from "../../ui/Term";
import styles from "./RewriteNotice.module.css";
import { useForcePushStore } from "./forcePushStore";

/** "올린 버전을 바꿨어요" 알림 줄(D-129). 이 브랜치에 표식이 있을 때만 보이고 덮어쓰며 올리기 창을 연다. */
export function RewriteNotice({ repo }: { repo: string }) {
  const m = useMessages();
  const allowed = (useAppSettings().data ?? DEFAULT_SETTINGS).allowPublishedRewrite;
  const plan = useForcePushPlan(repo, allowed);
  const openDialog = useForcePushStore((s) => s.openDialog);
  if (!allowed || plan.data?.rewrotePublished !== true) return null;
  return (
    <div className={styles.notice} role="status">
      <span>{m.forcePush.rewrote}</span>
      <Button variant="secondary" onClick={openDialog}>
        <Term id="force-push" focusable={false} />
      </Button>
    </div>
  );
}
