import type { KeyboardEvent } from "react";
import { SHORTCUTS, matches, type Shortcut, type ShortcutId } from "../../lib/shortcuts";
import { useMessages } from "../../messages";
import { Button } from "../../ui/Button";
import { DialogFrame } from "../../ui/DialogFrame";
import { Kbd } from "../../ui/Kbd";
import styles from "./ShortcutsDialog.module.css";
import { useShortcutsStore } from "./shortcutsStore";

const GROUPS: readonly Shortcut["scope"][] = ["global", "field", "fileList", "conflict", "history"];

/** 단축키 표(D-198). `SHORTCUTS`에서 만들어 빠짐이 없다. 열린 동안 전역 단축키는 쉬므로 Ctrl+/(닫기)는 여기서 받는다. */
export function ShortcutsDialog() {
  const m = useMessages();
  const { open, setOpen } = useShortcutsStore();
  if (!open) return null;
  const close = () => setOpen(false);
  const onKeyDown = (e: KeyboardEvent) => {
    if (matches("shortcutHelp", e.nativeEvent)) {
      e.preventDefault();
      close();
    }
  };
  const ids = Object.keys(SHORTCUTS) as ShortcutId[];
  return (
    <DialogFrame title={m.shortcutsTitle} onClose={close}>
      <div className={styles.groups} onKeyDown={onKeyDown}>
        {GROUPS.map((scope) => {
          const inGroup = ids.filter((id) => SHORTCUTS[id].scope === scope);
          return (
            <section key={scope} aria-label={m.shortcutGroup[scope]}>
              <h3 className={styles.groupTitle}>{m.shortcutGroup[scope]}</h3>
              <dl className={styles.rows}>
                {inGroup.map((id) => (
                  <div key={id} className={styles.row}>
                    <dt>{m.shortcutName[id]}</dt>
                    <dd>
                      <Kbd>{SHORTCUTS[id].label}</Kbd>
                    </dd>
                  </div>
                ))}
              </dl>
            </section>
          );
        })}
        <div className={styles.actions}>
          <Button variant="text" onClick={close}>
            {m.close}
          </Button>
        </div>
      </div>
    </DialogFrame>
  );
}
