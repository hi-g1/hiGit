import type { StationDialog } from "../ask";
import type { MenuCtx, MenuEnv } from "../items";
import { AmendDialog } from "./AmendDialog";
import { FilesDialog } from "./FilesDialog";
import { NewBranchDialog } from "./NewBranchDialog";
import { TagDialog } from "./TagDialog";

type Props = {
  dialog: StationDialog | null;
  onClose: () => void;
  env: MenuEnv;
  ctx: MenuCtx;
};

/** 역 메뉴가 연 대화상자를 그린다. */
export function StationDialogs({ dialog, onClose, env, ctx }: Props) {
  if (dialog === null) return null;
  switch (dialog.kind) {
    case "tag":
      return <TagDialog info={dialog.info} ctx={ctx} onClose={onClose} />;
    case "newBranch":
      return (
        <NewBranchDialog
          oid={dialog.info.oid}
          shortOid={dialog.info.shortOid}
          summary={dialog.info.summary}
          ctx={ctx}
          onClose={onClose}
        />
      );
    case "amend":
      return <AmendDialog info={dialog.info} stagedFiles={env.stagedFiles} ctx={ctx} onClose={onClose} />;
    case "files":
      return <FilesDialog info={dialog.info} ctx={ctx} onClose={onClose} />;
    // 작업 공간의 ForcePushDialog가 그린다.
    case "forcePush":
      return null;
  }
}
