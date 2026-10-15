import styles from "./Kbd.module.css";

/** 단축키 표시(옅은 코드 글꼴, design.md "굵은 글자 버튼"). */
export function Kbd({ children }: { children: string }) {
  return <kbd className={styles.kbd}>{children}</kbd>;
}
