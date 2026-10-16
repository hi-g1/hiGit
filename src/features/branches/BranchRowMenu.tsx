import * as Menu from "@radix-ui/react-dropdown-menu";
import { useRef } from "react";
import type { BranchRow } from "../../lib/api";
import { useMessages } from "../../messages";
import { Term } from "../../ui/Term";
import { Tooltip } from "../../ui/Tooltip";
import styles from "./BranchList.module.css";

type Props = {
  row: BranchRow;
  onRename: () => void;
  onDelete: () => void;
};

/**
 * 로컬 브랜치 줄 오른쪽 `…`(D-103): 이름 바꾸기·지우기. 지금 브랜치와 기본 브랜치는 지우지 않는다(흐린
 * 항목도 키보드로 닿고 까닭이 툴팁으로 뜬다). 줄 단추 밖의 형제라 Tab으로 닿는다.
 */
export function BranchRowMenu({ row, onRename, onDelete }: Props) {
  const m = useMessages();
  const protectedRow = row.isHead || row.isDefault;
  // 항목을 골랐으면 목록이 포커스를 정한다(이름 입력칸, 지운 뒤 찾기 칸). 그냥 닫으면 `…`로 돌아간다.
  const chose = useRef(false);
  const choose = (run: () => void) => () => {
    chose.current = true;
    run();
  };
  return (
    <Menu.Root modal={false}>
      <Menu.Trigger asChild>
        <button type="button" className={styles.rowMenu} aria-label={m.branchRowMenu(row.name)}>
          …
        </button>
      </Menu.Trigger>
      <Menu.Portal>
        <Menu.Content
          className={styles.menu}
          align="end"
          sideOffset={4}
          onCloseAutoFocus={(e) => {
            if (chose.current) e.preventDefault();
            chose.current = false;
          }}
        >
          <Menu.Item className={styles.menuItem} onSelect={choose(onRename)}>
            <Term id="branch-rename" focusable={false} />
          </Menu.Item>
          {protectedRow ? (
            <Tooltip content={m.errors.branchProtected}>
              <Menu.Item className={styles.menuItem} aria-disabled="true" onSelect={(e) => e.preventDefault()}>
                <Term id="branch-delete" focusable={false} />
              </Menu.Item>
            </Tooltip>
          ) : (
            <Menu.Item className={styles.menuItem} onSelect={choose(onDelete)}>
              <Term id="branch-delete" focusable={false} />
            </Menu.Item>
          )}
        </Menu.Content>
      </Menu.Portal>
    </Menu.Root>
  );
}
