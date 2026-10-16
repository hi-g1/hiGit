import * as Menu from "@radix-ui/react-dropdown-menu";
import { canReadText, copyText, readText } from "../lib/copy";
import { useMessages } from "../messages";
import { Kbd } from "./Kbd";
import styles from "./TextMenu.module.css";
import { Tooltip } from "./Tooltip";
import { useTextMenuStore, type TextMenuTarget } from "./textMenuStore";

/**
 * 입력칸·선택한 글자의 앱 메뉴(D-171, D-189). 웹뷰 기본 메뉴(새로 고침·검사 등) 대신 뜬다. 항목 글자는 상수이고
 * 클립보드는 `lib/copy`만 다룬다. 닫히면 포커스와 선택 범위가 입력칸으로 돌아간다.
 */
export function TextMenu() {
  const target = useTextMenuStore((s) => s.target);
  const close = useTextMenuStore((s) => s.close);
  if (!target) return null;
  return <Opened target={target} close={close} />;
}

function restore(target: TextMenuTarget) {
  if (target.kind !== "field") return;
  target.field.focus();
  target.field.setSelectionRange(target.start, target.end);
}

function Opened({ target, close }: { target: TextMenuTarget; close: () => void }) {
  const m = useMessages();
  const pasteBlocked = useTextMenuStore((s) => s.pasteBlocked);
  const blockPaste = useTextMenuStore((s) => s.blockPaste);
  const field = target.kind === "field" ? target.field : null;
  const hasSelection = target.kind === "selection" || target.start !== target.end;
  const editable = field !== null && !field.readOnly && !field.disabled;
  const canPaste = editable && !pasteBlocked && canReadText();

  // 입력칸 일은 포커스와 선택을 되살린 뒤 웹뷰의 편집 명령으로 한다(React 입력칸에 input 이벤트가 간다).
  const run = (command: "cut" | "copy" | "selectAll") => {
    restore(target);
    document.execCommand(command);
  };
  const copySelection = () => {
    if (target.kind === "selection") void copyText(target.text);
    else run("copy");
  };
  const paste = async () => {
    const text = await readText();
    if (text === null) {
      blockPaste();
      return;
    }
    restore(target);
    document.execCommand("insertText", false, text);
  };

  const item = (label: string, keys: string, enabled: boolean, onSelect: () => void, reason?: string) => {
    const body = (
      <>
        <span>{label}</span>
        <Kbd>{keys}</Kbd>
      </>
    );
    if (!enabled) {
      const node = (
        <Menu.Item className={styles.item} aria-disabled="true" onSelect={(e) => e.preventDefault()}>
          {body}
        </Menu.Item>
      );
      return reason ? (
        <Tooltip key={label} content={reason}>
          {node}
        </Tooltip>
      ) : (
        <span key={label}>{node}</span>
      );
    }
    return (
      <Menu.Item key={label} className={styles.item} onSelect={onSelect}>
        {body}
      </Menu.Item>
    );
  };

  return (
    <Menu.Root open onOpenChange={(open) => !open && close()} modal={false}>
      <Menu.Trigger asChild>
        <span className={styles.anchor} style={{ left: target.x, top: target.y }} aria-label={m.textMenu.label} tabIndex={-1} />
      </Menu.Trigger>
      <Menu.Portal>
        <Menu.Content
          className={styles.content}
          align="start"
          sideOffset={2}
          onCloseAutoFocus={(e) => {
            e.preventDefault();
            restore(target);
          }}
        >
          {field && item(m.textMenu.cut, "Ctrl+X", editable && hasSelection, () => run("cut"))}
          {item(m.textMenu.copy, "Ctrl+C", hasSelection, copySelection)}
          {field && item(m.textMenu.paste, "Ctrl+V", canPaste, () => void paste(), editable ? m.textMenu.pasteWithKeys : undefined)}
          {field && item(m.textMenu.selectAll, "Ctrl+A", field.value !== "", () => run("selectAll"))}
        </Menu.Content>
      </Menu.Portal>
    </Menu.Root>
  );
}
