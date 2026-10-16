import * as Menu from "@radix-ui/react-dropdown-menu";
import type { BranchRow } from "../../lib/api";
import { useMessages } from "../../messages";
import { Term } from "../../ui/Term";
import { Tooltip } from "../../ui/Tooltip";
import styles from "../branches/BranchList.module.css";
import { useRemoteBranchStore } from "./remoteBranchStore";

type Props = {
  row: BranchRow;
  remote: string;
  /** 원격 쪽 브랜치 이름(`origin/` 뺀 것). */
  branch: string;
  /** 고르면 목록을 닫는다(확인 창이 앞에 뜬다). */
  onChosen: () => void;
};

/** 원격 브랜치 줄 오른쪽 `…`(D-130): 원격 브랜치 이름 바꾸기·지우기. 원격 기본 브랜치는 흐리고 까닭은 툴팁. */
export function RemoteBranchMenu({ row, remote, branch, onChosen }: Props) {
  const m = useMessages();
  const open = useRemoteBranchStore((s) => s.open);
  const choose = (mode: "delete" | "rename") => () => {
    onChosen();
    open({ remote, branch, mode });
  };
  const item = (mode: "delete" | "rename") => {
    const label = <Term id={mode === "delete" ? "remote-branch-delete" : "remote-branch-rename"} focusable={false} />;
    if (row.isDefault) {
      return (
        <Tooltip content={m.errors.remoteBranchProtected}>
          <Menu.Item className={styles.menuItem} aria-disabled="true" onSelect={(e) => e.preventDefault()}>
            {label}
          </Menu.Item>
        </Tooltip>
      );
    }
    return (
      <Menu.Item className={styles.menuItem} onSelect={choose(mode)}>
        {label}
      </Menu.Item>
    );
  };
  return (
    <Menu.Root modal={false}>
      <Menu.Trigger asChild>
        <button type="button" className={styles.rowMenu} aria-label={m.branchRowMenu(`${remote}/${branch}`)}>
          …
        </button>
      </Menu.Trigger>
      <Menu.Portal>
        <Menu.Content className={styles.menu} align="end" sideOffset={4}>
          {item("rename")}
          {item("delete")}
        </Menu.Content>
      </Menu.Portal>
    </Menu.Root>
  );
}
