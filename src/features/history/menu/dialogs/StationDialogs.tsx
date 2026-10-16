import type { StationDialog } from "../ask";
import type { MenuCtx, MenuEnv } from "../items";
import { NewBranchDialog } from "./NewBranchDialog";
import { TagDialog } from "./TagDialog";

type Props = {
  dialog: StationDialog | null;
  onClose: () => void;
  env: MenuEnv;
  ctx: MenuCtx;
};

/** 역 메뉴가 연 대화상자를 그린다. */
export function StationDialogs({ dialog, onClose, ctx }: Props) {
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
    default:
      return null;
  }
}
