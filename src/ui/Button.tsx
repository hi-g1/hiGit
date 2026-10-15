import { forwardRef, type ButtonHTMLAttributes } from "react";
import styles from "./Button.module.css";

type ButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & {
  /**
   * primary: 화면마다 하나(--primary 채움, D-026). secondary: 테두리. text: 글자만(본문색).
   * danger: 되돌릴 수 없는 일의 확인 대화상자에서만(--deleted 채움, design.md "위험 버튼").
   */
  variant?: "primary" | "secondary" | "text" | "danger";
};

/** ref를 넘긴다(툴팁·팝오버의 `asChild` 트리거가 위치를 잡으려면 필요하다). */
export const Button = forwardRef<HTMLButtonElement, ButtonProps>(function Button(
  { variant = "secondary", className, type = "button", ...rest },
  ref,
) {
  const classes = [styles.button, styles[variant], className].filter(Boolean).join(" ");
  return <button ref={ref} type={type} className={classes} {...rest} />;
});
