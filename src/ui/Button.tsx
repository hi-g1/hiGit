import { forwardRef, useId, type ButtonHTMLAttributes, type MouseEvent } from "react";
import styles from "./Button.module.css";
import { Tooltip } from "./Tooltip";

type ButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & {
  /**
   * primary: 화면마다 하나(--primary 채움, D-026). secondary: 테두리. text: 글자만(본문색).
   * danger: 되돌릴 수 없는 일의 확인 대화상자에서만(--deleted 채움, design.md "위험 버튼").
   */
  variant?: "primary" | "secondary" | "text" | "danger";
  /**
   * 지금 쓸 수 없는 까닭(D-191). 있으면 `disabled` 대신 `aria-disabled`로 그려 키보드로도 닿고, 까닭이 툴팁과
   * `aria-describedby`로 나오며 누름은 무시한다. 진행 중 꺼짐은 이것 대신 `disabled`를 쓴다.
   */
  disabledReason?: string | null;
  /** 진행 중(D-193): 글자는 그대로 두고 오른쪽에 작은 진행 표시, `aria-busy`, 누름은 막는다. */
  busy?: boolean;
};

/** ref를 넘긴다(툴팁·팝오버의 `asChild` 트리거가 위치를 잡으려면 필요하다). */
export const Button = forwardRef<HTMLButtonElement, ButtonProps>(function Button(
  { variant = "secondary", className, type = "button", disabledReason, busy = false, onClick, children, ...rest },
  ref,
) {
  const reasonId = useId();
  const classes = [styles.button, styles[variant], className].filter(Boolean).join(" ");
  if (busy) {
    return (
      <button ref={ref} type={type} className={classes} {...rest} disabled aria-busy="true">
        {children}
        <span className={styles.spinner} aria-hidden="true" />
      </button>
    );
  }
  if (!disabledReason) {
    return (
      <button ref={ref} type={type} className={classes} onClick={onClick} {...rest}>
        {children}
      </button>
    );
  }
  const ignore = (e: MouseEvent<HTMLButtonElement>) => e.preventDefault();
  return (
    <>
      <Tooltip content={disabledReason}>
        <button
          ref={ref}
          type={type}
          className={classes}
          {...rest}
          disabled={false}
          aria-disabled="true"
          aria-describedby={reasonId}
          onClick={ignore}
        >
          {children}
        </button>
      </Tooltip>
      <span id={reasonId} hidden>
        {disabledReason}
      </span>
    </>
  );
});
