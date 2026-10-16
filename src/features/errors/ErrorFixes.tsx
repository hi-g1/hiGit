import type { ReactNode } from "react";
import type { ErrorCode } from "../../lib/api";
import { useMessages } from "../../messages";
import { useRepoStore } from "../../state/repoStore";
import { Button } from "../../ui/Button";
import type { ErrorActions } from "../../ui/errorActions";
import { RemoteRefusedFix } from "../github/RemoteRefusedFix";
import { WorkflowFix } from "../github/WorkflowFix";
import { IdentityFix } from "../identity/IdentityFix";
import { useSettingsStore } from "../settings/settingsStore";
import { ERROR_FIX, type FixKind } from "./fixes";

function ConnectFix({ again }: { again: boolean }) {
  const m = useMessages();
  const openSettings = useSettingsStore((s) => s.openSettings);
  return (
    <Button variant="secondary" onClick={() => openSettings(true)}>
      {again ? m.pr.reconnect : m.settings.connect}
    </Button>
  );
}

function HistoryFix() {
  const m = useMessages();
  const setView = useRepoStore((s) => s.setView);
  const open = useRepoStore((s) => s.repo !== null);
  if (!open) return null;
  return (
    <Button variant="secondary" onClick={() => setView("history")}>
      {m.fix.openHistory}
    </Button>
  );
}

/** 해결 버튼 종류 → 버튼. `none`은 버튼이 없다. */
const RENDER: Record<Exclude<FixKind, "none">, ReactNode> = {
  connect: <ConnectFix again={false} />,
  reconnect: <ConnectFix again />,
  setIdentity: <IdentityFix />,
  addWorkflow: <WorkflowFix />,
  branchSettings: <RemoteRefusedFix />,
  openHistory: <HistoryFix />,
};

/** 앱 맨 위가 `ErrorActionsContext`에 넣는 오류 코드별 공용 해결 버튼(D-194). `ERROR_FIX` 표에서 만든다. */
export function errorActions(): ErrorActions {
  const out: ErrorActions = {};
  for (const [code, kind] of Object.entries(ERROR_FIX) as [ErrorCode, FixKind][]) {
    if (kind !== "none") out[code] = RENDER[kind];
  }
  return out;
}
