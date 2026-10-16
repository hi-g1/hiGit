import * as Menu from "@radix-ui/react-dropdown-menu";
import type { BranchRow } from "../../lib/api";
import { useMessages } from "../../messages";
import { Term } from "../../ui/Term";
import { Tooltip } from "../../ui/Tooltip";
import styles from "./BranchList.module.css";

type Props = {
  row: BranchRow;
  /** 고정했는지(D-203). */
  pinned: boolean;
  onPin: (pinned: boolean) => void;
  /** 열림은 목록이 정한다(찾기 칸의 메뉴 키로도 연다, D-201). */
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onRename: () => void;
  onDelete: () => void;
};

/**
 * 로컬 브랜치 줄 오른쪽 `…`(D-103): 이름 바꾸기·지우기. 지금 브랜치와 기본 브랜치는 지우지 않는다(흐린
 * 항목도 키보드로 닿고 까닭이 툴팁으로 뜬다). `…`는 마우스용이고(Tab 정지 아님, 읽기 프로그램에서 숨김)
 * 키보드는 찾기 칸에서 고른 줄의 메뉴 키·Shift+F10으로 연다(D-201).
 */
export function BranchRowMenu({ row, pinned, onPin, open, onOpenChange, onRename, onDelete }: Props) {
  const m = useMessages();
  const protectedRow = row.isHead || row.isDefault;
  return (
    <Menu.Root modal={false} open={open} onOpenChange={onOpenChange}>
      <Menu.Trigger asChild>
        <button type="button" className={styles.rowMenu} aria-label={m.branchRowMenu(row.name)} aria-hidden="true" tabIndex={-1}>
          …
        </button>
      </Menu.Trigger>
      <Menu.Portal>
        <Menu.Content
          className={styles.menu}
          align="end"
          sideOffset={4}
          // 포커스는 목록이 정한다(이름 입력칸, 지운 뒤나 그냥 닫으면 찾기 칸).
          onCloseAutoFocus={(e) => e.preventDefault()}
        >
          <Menu.Item className={styles.menuItem} onSelect={() => onPin(!pinned)}>
            {pinned ? m.unpin : m.pin}
          </Menu.Item>
          <Menu.Item className={styles.menuItem} onSelect={onRename}>
            <Term id="branch-rename" focusable={false} />
          </Menu.Item>
          {protectedRow ? (
            <Tooltip content={m.errors.branchProtected}>
              <Menu.Item className={styles.menuItem} aria-disabled="true" onSelect={(e) => e.preventDefault()}>
                <Term id="branch-delete" focusable={false} />
              </Menu.Item>
            </Tooltip>
          ) : (
            <Menu.Item className={styles.menuItem} onSelect={onDelete}>
              <Term id="branch-delete" focusable={false} />
            </Menu.Item>
          )}
        </Menu.Content>
      </Menu.Portal>
    </Menu.Root>
  );
}
