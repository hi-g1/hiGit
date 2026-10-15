import { getTerm, type TermId } from "../glossary";
import { useUiStore } from "../state/uiStore";
import styles from "./Term.module.css";
import { Tooltip } from "./Tooltip";

type TermProps = {
  id: TermId;
};

/**
 * 화면에 Git 용어를 쓸 때는 항상 이 부품을 쓴다(D-006).
 * 본문에는 쉬운 이름, 툴팁에는 원래 이름 · 명령 · 설명.
 * "Git 원래 이름으로 보기"에서는 본문과 툴팁 첫 줄이 서로 바뀐다.
 */
export function Term({ id }: TermProps) {
  const termDisplay = useUiStore((s) => s.termDisplay);
  const locale = useUiStore((s) => s.locale);
  const term = getTerm(id, locale);

  const showGit = termDisplay === "git";
  const body = showGit ? term.gitName : term.friendly;
  const heading = showGit ? term.friendly : term.gitName;

  return (
    <Tooltip
      content={
        <span className={styles.tip}>
          <span className={styles.heading}>{heading}</span>
          {term.command !== null && <code className={styles.command}>{term.command}</code>}
          <span className={styles.description}>{term.description}</span>
        </span>
      }
    >
      <span className={styles.term} tabIndex={0}>
        {body}
      </span>
    </Tooltip>
  );
}
