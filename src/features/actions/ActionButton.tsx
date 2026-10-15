import { useMutation } from "@tanstack/react-query";
import { isTermId } from "../../glossary";
import { api, type ActionInput, type ActionResult } from "../../lib/api";
import { useActions } from "../../lib/queries";
import { Button } from "../../ui/Button";
import { ErrorNotice } from "../../ui/ErrorNotice";
import { Term } from "../../ui/Term";

type ActionButtonProps = {
  /** 실행할 액션과 입력. 액션 id는 `input.id`다(D-020). */
  input: ActionInput;
  folder: string;
  variant?: "primary" | "secondary";
  onDone: (result: ActionResult) => void;
};

/**
 * 레지스트리 메타(action_list)로 만드는 액션 버튼. 이름은 glossary에서 온다.
 * 미리보기가 필요한 caution 이상 액션은 반복 2에서 미리보기 흐름과 함께 붙인다(D-011).
 */
export function ActionButton({ input, folder, variant = "secondary", onDone }: ActionButtonProps) {
  const actions = useActions();
  const meta = actions.data?.find((a) => a.id === input.id);
  const run = useMutation({
    mutationFn: () => api.executeAction(folder, input),
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
