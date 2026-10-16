import { useMessages } from "../../messages";
import { Button } from "../../ui/Button";
import { useSettingsStore } from "../settings/settingsStore";

/** 워크플로 권한이 없어 올리지 못한 오류 줄의 단추(D-135). 설정을 열고 그 권한으로 다시 연결한다. */
export function WorkflowFix() {
  const m = useMessages();
  const openWorkflowConnect = useSettingsStore((s) => s.openWorkflowConnect);
  return (
    <Button variant="secondary" onClick={openWorkflowConnect}>
      {m.settings.addWorkflow}
    </Button>
  );
}
