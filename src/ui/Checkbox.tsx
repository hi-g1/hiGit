import { forwardRef, useEffect, useImperativeHandle, useRef, type InputHTMLAttributes } from "react";
import styles from "./Checkbox.module.css";

type CheckboxProps = Omit<InputHTMLAttributes<HTMLInputElement>, "type"> & {
  /** 일부만 켜진 상태(세 상태 체크 상자). */
  indeterminate?: boolean;
};

/** OS 기본 모양 대신 토큰 색을 쓰는 체크 상자. 켜짐은 --primary(design.md). */
export const Checkbox = forwardRef<HTMLInputElement, CheckboxProps>(function Checkbox(
  { indeterminate = false, className, ...rest },
  ref,
) {
  const inner = useRef<HTMLInputElement>(null);
  useImperativeHandle(ref, () => inner.current!);
  useEffect(() => {
    if (inner.current) inner.current.indeterminate = indeterminate;
  }, [indeterminate]);
  return (
    <input
      ref={inner}
      type="checkbox"
      className={[styles.box, className].filter(Boolean).join(" ")}
      {...rest}
    />
  );
});
