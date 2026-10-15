import type { ReactNode } from "react";
import styles from "./Panel.module.css";

/** 제목이 있는 패널. 제목에는 보통 <Term>이 들어간다. */
export function Panel({ title, aside, children }: { title: ReactNode; aside?: ReactNode; children: ReactNode }) {
  return (
    <section className={styles.panel}>
      <header className={styles.header}>
        <h2 className={styles.title}>{title}</h2>
        {aside && <span className={styles.aside}>{aside}</span>}
      </header>
      <div className={styles.body}>{children}</div>
    </section>
  );
}
