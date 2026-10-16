import * as Dialog from "@radix-ui/react-dialog";
import type { ReactNode, RefObject } from "react";
import styles from "./DialogFrame.module.css";

type Props = {
  title: ReactNode;
  /** 제목 아래 한 줄(역의 짧은 해시와 이름, `feature → main` 등). 저장소 글자는 React 글자로만. */
  subtitle?: ReactNode;
  onClose: () => void;
  children: ReactNode;
  /** 폭: 기본 440px, 넓게 560px. */
  size?: "narrow" | "wide";
  className?: string;
  /** 열릴 때 포커스를 둘 곳(되돌릴 수 없는 확인 창은 안전한 "닫기"). 없으면 첫 컨트롤. */
  initialFocus?: RefObject<HTMLElement | null>;
};

/**
 * 대화상자 틀(Radix Dialog). 열린 동안 키는 대화상자 밖으로 나가지 않고(`aria-modal`이면 `useShortcut`도
 * 쉰다) 뒤의 전역 단축키(Ctrl+Z·Ctrl+H 등)가 돌지 않는다. Esc와 바깥 누름은 닫기. 닫은 뒤 포커스는 부른 쪽이 돌려준다.
 */
export function DialogFrame({ title, subtitle, onClose, children, size, className, initialFocus }: Props) {
  const width = size === "wide" ? styles.wide : "";
  return (
    <Dialog.Root open onOpenChange={(open) => !open && onClose()}>
      <Dialog.Portal>
        <Dialog.Overlay className={styles.overlay} />
        <Dialog.Content
          className={`${styles.dialog} ${width} ${className ?? ""}`}
          aria-describedby={undefined}
          aria-modal="true"
          onKeyDown={(e) => e.stopPropagation()}
          onOpenAutoFocus={(e) => {
            if (!initialFocus) return;
            e.preventDefault();
            initialFocus.current?.focus();
          }}
          // 열기 전 포커스 자리(메뉴 항목 등)는 이미 사라졌을 수 있다.
          onCloseAutoFocus={(e) => e.preventDefault()}
        >
          <Dialog.Title className={styles.title}>{title}</Dialog.Title>
          {subtitle !== undefined && <p className={styles.subtitle}>{subtitle}</p>}
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
