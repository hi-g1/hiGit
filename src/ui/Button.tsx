import type { ButtonHTMLAttributes } from "react";
import styles from "./Button.module.css";

type ButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & {
  /** primary: 화면마다 하나(--accent 채움). secondary: 테두리. text: 글자만. */
  variant?: "primary" | "secondary" | "text";
};

export function Button({ variant = "secondary", className, type = "button", ...rest }: ButtonProps) {
  const classes = [styles.button, styles[variant], className].filter(Boolean).join(" ");
  return <button type={type} className={classes} {...rest} />;
}
