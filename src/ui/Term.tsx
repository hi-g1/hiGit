import type { ReactElement } from "react";
import { getTerm, type TermId } from "../glossary";
import { useUiStore } from "../state/uiStore";
import styles from "./Term.module.css";
import { Tooltip } from "./Tooltip";

/** 설정("Git 원래 이름으로 보기")과 언어를 따른 용어 이름. 접근성 이름처럼 글자만 필요할 때 쓴다. */
export function useTermLabel(id: TermId): string {
  const termDisplay = useUiStore((s) => s.termDisplay);
  const locale = useUiStore((s) => s.locale);
  const term = getTerm(id, locale);
  return termDisplay === "git" ? term.gitName : term.friendly;
}

/** 용어 툴팁(원래 이름 · 명령 · 설명)을 다른 부품(체크 상자 등)에 씌운다. */
export function TermTooltip({ id, children }: { id: TermId; children: ReactElement }) {
  const termDisplay = useUiStore((s) => s.termDisplay);
  const locale = useUiStore((s) => s.locale);
  const term = getTerm(id, locale);
  const heading = termDisplay === "git" ? term.friendly : term.gitName;

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
      {children}
    </Tooltip>
  );
}

type TermProps = {
  id: TermId;
  /**
   * 버튼처럼 이미 포커스되는 요소 안에 넣을 때는 false. 포커스가 겹치지 않게 하고,
   * 툴팁은 호버로 뜬다.
   */
  focusable?: boolean;
};

/**
 * 화면에 Git 용어를 쓸 때는 항상 이 부품을 쓴다(D-006).
 * 본문에는 쉬운 이름, 툴팁에는 원래 이름 · 명령 · 설명.
 * "Git 원래 이름으로 보기"에서는 본문과 툴팁 첫 줄이 서로 바뀐다.
 */
export function Term({ id, focusable = true }: TermProps) {
  const label = useTermLabel(id);
  return (
    <TermTooltip id={id}>
      <span className={styles.term} tabIndex={focusable ? 0 : undefined}>
        {label}
      </span>
    </TermTooltip>
  );
}
