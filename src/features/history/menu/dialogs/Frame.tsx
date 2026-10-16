import * as Dialog from "@radix-ui/react-dialog";
import type { ReactNode } from "react";
import styles from "./Dialogs.module.css";

type Props = {
  title: ReactNode;
  /** 제목 아래 한 줄(역의 짧은 해시와 이름). 저장소 글자는 React 글자로만. */
  station: ReactNode;
  onClose: () => void;
  children: ReactNode;
  className?: string;
};

/**
 * 역 메뉴 대화상자의 틀(Radix Dialog, 폭 440px). 열린 동안 키는 대화상자 밖으로 나가지 않는다: 뒤의
 * 전역 단축키(Ctrl+Z·Ctrl+H 등)가 돌지 않는다. Esc와 바깥 누름은 닫기.
 */
export function Frame({ title, station, onClose, children, className }: Props) {
  return (
    <Dialog.Root open onOpenChange={(open) => !open && onClose()}>
      <Dialog.Portal>
        <Dialog.Overlay className={styles.overlay} />
        <Dialog.Content
          className={`${styles.dialog} ${className ?? ""}`}
          aria-describedby={undefined}
          onKeyDown={(e) => e.stopPropagation()}
          // 열기 전 포커스 자리(메뉴 항목)는 이미 사라졌다. 닫은 뒤 포커스는 부른 쪽이 돌려준다.
          onCloseAutoFocus={(e) => e.preventDefault()}
        >
          <Dialog.Title className={styles.title}>{title}</Dialog.Title>
          <p className={styles.station}>{station}</p>
          {children}
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}

/** 한글 조합 중 Enter는 글자를 확정하는 키라서 실행하지 않는다. */
export function composing(e: { nativeEvent: KeyboardEvent }): boolean {
  return e.nativeEvent.isComposing || e.nativeEvent.keyCode === 229;
}
