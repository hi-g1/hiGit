import * as Menu from "@radix-ui/react-dropdown-menu";
import { useMessages } from "../../messages";
import { Term } from "../../ui/Term";
import styles from "../branches/BranchList.module.css";
import { useRemoteBranchStore } from "./remoteBranchStore";

type Props = {
  remote: string;
  /** 원격 쪽 브랜치 이름(`origin/` 뺀 것). */
  branch: string;
  /** 고르면 목록을 닫는다(확인 창이 앞에 뜬다). */
  onChosen: () => void;
  /** 열림은 목록이 정한다(찾기 칸의 메뉴 키로도 연다, D-201). */
  open: boolean;
  onOpenChange: (open: boolean) => void;
};

/**
 * 원격 브랜치 줄 오른쪽 `…`(D-130): 원격 브랜치 이름 바꾸기·지우기. 기본 브랜치 여부는 확인 창이 정한다(서버 기본은
 * 까닭과 꺼진 위험 버튼, 이름만 같은 것은 이름 입력, D-183).
 */
export function RemoteBranchMenu({ remote, branch, onChosen, open: menuOpen, onOpenChange }: Props) {
  const m = useMessages();
  const open = useRemoteBranchStore((s) => s.open);
  const choose = (mode: "delete" | "rename") => () => {
    onChosen();
    open({ remote, branch, mode });
  };
  const item = (mode: "delete" | "rename") => {
    const label = <Term id={mode === "delete" ? "remote-branch-delete" : "remote-branch-rename"} focusable={false} />;
    // 기본 브랜치인지(서버 기본은 막고, 이름만 같은 것은 이름 입력) 확인 창의 계획 조회가 정한다(D-183).
    return (
      <Menu.Item className={styles.menuItem} onSelect={choose(mode)}>
        {label}
      </Menu.Item>
    );
  };
  return (
    <Menu.Root modal={false} open={menuOpen} onOpenChange={onOpenChange}>
      <Menu.Trigger asChild>
        {/* 마우스용(D-201). 키보드는 찾기 칸에서 메뉴 키 */}
        <button
          type="button"
          className={styles.rowMenu}
          aria-label={m.branchRowMenu(`${remote}/${branch}`)}
          aria-hidden="true"
          tabIndex={-1}
        >
          …
        </button>
      </Menu.Trigger>
      <Menu.Portal>
        <Menu.Content className={styles.menu} align="end" sideOffset={4} onCloseAutoFocus={(e) => e.preventDefault()}>
          {item("rename")}
          {item("delete")}
        </Menu.Content>
      </Menu.Portal>
    </Menu.Root>
  );
}
