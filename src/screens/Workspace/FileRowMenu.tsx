import * as Menu from "@radix-ui/react-dropdown-menu";
import type { ChangedFile } from "../../lib/api";
import { copyText } from "../../lib/copy";
import { useMessages } from "../../messages";
import { Term } from "../../ui/Term";
import { useToastStore } from "../../ui/toastStore";
import menuStyles from "../../features/history/menu/StationMenu.module.css";

type Props = {
  file: ChangedFile | null;
  /** 파일 목록 안의 좌표(누른 자리나 줄의 아래쪽). */
  anchor: { x: number; y: number };
  onClose: () => void;
  onDiscard: (file: ChangedFile) => void;
};

/**
 * 고친 파일 줄의 오른쪽 클릭 메뉴(계획 011 R13). 경로 복사와 이 파일 고친 것 버리기(기존 확인 흐름).
 * 메뉴 키·Shift+F10으로도 연다. 역 메뉴와 같은 모양을 쓴다.
 */
export function FileRowMenu({ file, anchor, onClose, onDiscard }: Props) {
  const m = useMessages();
  const show = useToastStore((s) => s.show);
  return (
    <Menu.Root open={file !== null} onOpenChange={(open) => !open && onClose()} modal={false}>
      <Menu.Trigger asChild>
        <span className={menuStyles.anchor} style={{ left: anchor.x, top: anchor.y }} aria-label={m.fileMenu} tabIndex={-1} />
      </Menu.Trigger>
      <Menu.Portal>
        <Menu.Content className={menuStyles.content} align="start" sideOffset={4}>
          <Menu.Item
            className={menuStyles.item}
            onSelect={() => {
              if (!file) return;
              void copyText(file.path).then((ok) => show(ok ? m.copied : m.copyFailed));
            }}
          >
            {m.copyPath}
          </Menu.Item>
          <Menu.Item className={menuStyles.item} onSelect={() => file && onDiscard(file)}>
            {m.thisFile} <Term id="discard" focusable={false} />
          </Menu.Item>
        </Menu.Content>
      </Menu.Portal>
    </Menu.Root>
  );
}
