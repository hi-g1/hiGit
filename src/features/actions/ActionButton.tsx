import { useMutation } from "@tanstack/react-query";
import { isTermId } from "../../glossary";
import { api, type ActionResult } from "../../lib/api";
import { useActions } from "../../lib/queries";
import { Button } from "../../ui/Button";
import { ErrorNotice } from "../../ui/ErrorNotice";
import { Term } from "../../ui/Term";

type ActionButtonProps = {
  actionId: string;
  folder: string;
  variant?: "primary" | "secondary";
  onDone: (result: ActionResult) => void;
};

/**
 * 레지스트리 메타(action_list)로 만드는 액션 버튼. 이름은 glossary에서 온다.
 * 미리보기가 필요한 caution 이상 액션은 반복 2에서 미리보기 흐름과 함께 붙인다(D-011).
 */
export function ActionButton({ actionId, folder, variant = "secondary", onDone }: ActionButtonProps) {
  const actions = useActions();
  const meta = actions.data?.find((a) => a.id === actionId);
  const run = useMutation({
    mutationFn: () => api.executeAction(folder, actionId),
    onSuccess: onDone,
  });

  if (!meta || !isTermId(meta.glossaryId)) return null;
  if (meta.risk === "caution" || meta.risk === "danger") return null;

  return (
    <>
      <Button variant={variant} disabled={run.isPending} onClick={() => run.mutate()}>
        <Term id={meta.glossaryId} focusable={false} />
      </Button>
      {run.error && <ErrorNotice error={run.error} />}
    </>
  );
}
