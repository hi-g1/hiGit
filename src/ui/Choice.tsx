import { useRef, type KeyboardEvent, type ReactNode } from "react";
import { choiceIndex, choiceKey } from "../lib/choiceKeys";
import styles from "./Choice.module.css";

export type ChoiceOption<T extends string> = { value: T; label: ReactNode };

type Props<T extends string> = {
  /** 묶음 이름(설정 줄 이름). */
  label: string;
  value: T;
  options: readonly ChoiceOption<T>[];
  onChange: (value: T) => void;
};

/**
 * 몇 가지 중 하나를 고르는 글자 버튼 묶음(D-160). 고른 것은 굵게. 방향키로 옮기면 바로 고른다(radiogroup).
 * Tab은 고른 것 하나에만 멈춘다.
 */
export function Choice<T extends string>({ label, value, options, onChange }: Props<T>) {
  const buttons = useRef<(HTMLButtonElement | null)[]>([]);
  const onKeyDown = (e: KeyboardEvent<HTMLButtonElement>, at: number) => {
    const move = choiceKey({ ...e, isComposing: e.nativeEvent.isComposing });
    if (!move) return;
    e.preventDefault();
    const next = choiceIndex(move, at, options.length);
    const option = options[next];
    if (!option) return;
    onChange(option.value);
    buttons.current[next]?.focus();
  };
  return (
    <div role="radiogroup" aria-label={label} className={styles.group}>
      {options.map((o, i) => {
        const checked = o.value === value;
        return (
          <button
            key={o.value}
            ref={(el) => {
              buttons.current[i] = el;
            }}
            type="button"
            role="radio"
            aria-checked={checked}
            tabIndex={checked ? 0 : -1}
            className={`${styles.option} ${checked ? styles.checked : ""}`}
            onClick={() => onChange(o.value)}
            onKeyDown={(e) => onKeyDown(e, i)}
          >
            {o.label}
          </button>
        );
      })}
    </div>
  );
}
