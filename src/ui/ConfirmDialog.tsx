import { useRef, type ReactNode } from "react";
import { Button } from "./Button";
import styles from "./ConfirmDialog.module.css";
import { DialogFrame } from "./DialogFrame";

type Props = {
  open: boolean;
  title: ReactNode;
  /** 대상(파일 이름·경로 등), 코드 글꼴 한 줄. */
  subtitle?: ReactNode;
  children?: ReactNode;
  dangerLabel: ReactNode;
  closeLabel: ReactNode;
  /** 실행 중이면 위험 버튼을 끈다. */
  pending?: boolean;
  /** 할 수 없는 상태면 위험 버튼을 끈다(까닭은 본문에). */
  dangerDisabled?: boolean;
  /** 본문이 길면 넓게(560px). */
  size?: "narrow" | "wide";
  onConfirm: () => void;
  onClose: () => void;
};

/**
 * 되돌릴 수 없는 일의 확인 창(D-043, design.md "확인 대화상자"). 포커스는 안전한 "닫기"(주 버튼)에 있어
 * Enter가 위험 버튼이 되지 않는다. 열린 동안 전역 단축키는 쉰다(DialogFrame).
 */
export function ConfirmDialog({
  open,
  title,
  subtitle,
  children,
  dangerLabel,
  closeLabel,
  pending,
  dangerDisabled,
  size,
  onConfirm,
  onClose,
}: Props) {
  const close = useRef<HTMLButtonElement>(null);
  if (!open) return null;
  return (
    <DialogFrame title={title} onClose={onClose} initialFocus={close} size={size}>
      {subtitle !== undefined && <p className={styles.target}>{subtitle}</p>}
      {children}
      <div className={styles.actions}>
        <Button variant="danger" disabled={pending || dangerDisabled} onClick={onConfirm}>
          {dangerLabel}
        </Button>
        <Button ref={close} variant="primary" onClick={onClose}>
          {closeLabel}
        </Button>
      </div>
    </DialogFrame>
  );
}
