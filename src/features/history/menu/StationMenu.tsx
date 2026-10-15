import * as Menu from "@radix-ui/react-dropdown-menu";
import { useMessages } from "../../../messages";
import type { StationInfo } from "./info";
import { STATION_ITEMS, type MenuCtx, type MenuEnv, type StationMenuItem } from "./items";
import styles from "./StationMenu.module.css";

type Props = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  info: StationInfo | null;
  env: MenuEnv;
  ctx: MenuCtx;
  items?: readonly StationMenuItem[];
  /** 메뉴를 붙일 자리(목록 안의 좌표). 목록과 함께 스크롤된다. */
  anchor: { x: number; y: number };
  /** 닫히면 포커스를 돌려줄 곳(목록). */
  returnFocus: () => void;
  /** 이 안을 누른 것은 바깥 누름으로 보지 않는다(목록이 스스로 고르고 연다). */
  isInside: (node: Node) => boolean;
};

/**
 * 역 메뉴(r-log.png, D-092). 항목 표에서 보이는 것만 그린다. "더 보기"는 그 묶음에 보이는 항목이 있을
 * 때만. 비모달이라 열린 채 스크롤하면 목록이 닫는다. Esc는 메뉴만 닫는다(Radix가 그 키를 처리했다고
 * 표시하므로 버전 기록 화면은 홈으로 가지 않는다).
 */
export function StationMenu({ open, onOpenChange, info, env, ctx, items = STATION_ITEMS, anchor, returnFocus, isInside }: Props) {
  const m = useMessages();
  if (!info) return null;
  const visible = items
    .map((item) => ({ item, state: item.availability(info, env) }))
    .filter(({ state }) => state !== "hide");
  const main = visible.filter(({ item }) => item.group === "main");
  const more = visible.filter(({ item }) => item.group === "more");
  const render = ({ item, state }: (typeof visible)[number]) => {
    const disabled = typeof state === "object" ? state.disabled(m) : null;
    return (
      <Menu.Item
        key={item.id}
        className={styles.item}
        disabled={disabled !== null}
        title={disabled ?? undefined}
        onSelect={() => void item.run(info, ctx)}
      >
        {item.label(m)}
      </Menu.Item>
    );
  };
  return (
    <Menu.Root open={open} onOpenChange={onOpenChange} modal={false}>
      <Menu.Trigger asChild>
        {/* 메뉴의 이름은 이 자리의 이름에서 온다(Radix가 aria-labelledby로 잇는다). */}
        <span
          className={styles.anchor}
          style={{ left: anchor.x, top: anchor.y }}
          aria-label={m.stationMenu}
          tabIndex={-1}
        />
      </Menu.Trigger>
      <Menu.Portal>
        <Menu.Content
          className={styles.content}
          align="start"
          sideOffset={4}
          onCloseAutoFocus={(e) => {
            e.preventDefault();
            returnFocus();
          }}
          onPointerDownOutside={(e) => {
            if (e.target instanceof Node && isInside(e.target)) e.preventDefault();
          }}
        >
          {main.map(render)}
          {more.length > 0 && (
            <>
              <Menu.Separator className={styles.separator} />
              <Menu.Sub>
                <Menu.SubTrigger className={styles.item}>{m.moreItems}</Menu.SubTrigger>
                <Menu.Portal>
                  <Menu.SubContent className={styles.content} sideOffset={4}>
                    {more.map(render)}
                  </Menu.SubContent>
                </Menu.Portal>
              </Menu.Sub>
            </>
          )}
        </Menu.Content>
      </Menu.Portal>
    </Menu.Root>
  );
}
