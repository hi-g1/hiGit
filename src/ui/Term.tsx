import type { ReactElement } from "react";
import { getTerm, type TermId } from "../glossary";
import { SHORTCUTS, type ShortcutId } from "../lib/shortcuts";
import { useUiStore } from "../state/uiStore";
import { Kbd } from "./Kbd";
import styles from "./Term.module.css";
import { Tooltip } from "./Tooltip";

/** 용어 툴팁 지연. 글자 버튼 위를 지나갈 때마다 뜨지 않게 다른 툴팁(400ms)보다 길다(D-140, D-154). */
export const TERM_TOOLTIP_DELAY_MS = 600;

/** 설정("Git 원래 이름으로 보기")과 언어를 따른 용어 이름. 접근성 이름처럼 글자만 필요할 때 쓴다. */
export function useTermLabel(id: TermId): string {
  const termDisplay = useUiStore((s) => s.termDisplay);
  const locale = useUiStore((s) => s.locale);
  const term = getTerm(id, locale);
  return termDisplay === "git" ? term.gitName : term.friendly;
}

/**
 * 용어 툴팁(원래 이름 · 명령 · 설명)을 다른 부품(체크 상자 등)에 씌운다. 설정 "용어 설명 보기"를 끄면
 * 툴팁 없이 그대로 돌려준다(D-140).
 */
export function TermTooltip({
  id,
  shortcut,
  children,
}: {
  id: TermId;
  /** 툴팁 맨 아래에 보일 단축키(D-198). 용어 설명을 꺼도 단축키만은 보인다. */
  shortcut?: ShortcutId;
  children: ReactElement;
}) {
  const termDisplay = useUiStore((s) => s.termDisplay);
  const locale = useUiStore((s) => s.locale);
  const termHelp = useUiStore((s) => s.termHelp);
  const term = getTerm(id, locale);
  const heading = termDisplay === "git" ? term.friendly : term.gitName;
  // English에서 화면 이름과 원래 이름이 같으면("Push"·"push") 이름 줄을 뺀다(D-139).
  const showHeading = term.friendly.toLowerCase() !== term.gitName.toLowerCase();
  const keys = shortcut && <Kbd>{SHORTCUTS[shortcut].label}</Kbd>;
  if (!termHelp) return keys ? <Tooltip content={keys}>{children}</Tooltip> : children;

  return (
    <Tooltip
      delay={TERM_TOOLTIP_DELAY_MS}
      content={
        <span className={styles.tip}>
          {showHeading && <span className={styles.heading}>{heading}</span>}
          {term.command !== null && <code className={styles.command}>{term.command}</code>}
          <span className={styles.description}>{term.description}</span>
          {keys}
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
  /** 툴팁에 보일 단축키(D-198). */
  shortcut?: ShortcutId;
};

/**
 * 화면에 Git 용어를 쓸 때는 항상 이 부품을 쓴다(D-006).
 * 본문에는 쉬운 이름, 툴팁에는 원래 이름 · 명령 · 설명. 글자에는 밑줄·점선 없이 툴팁만(D-140).
 * "Git 원래 이름으로 보기"에서는 본문과 툴팁 첫 줄이 서로 바뀐다.
 * "용어 설명 보기"를 끄면 툴팁이 없으므로 포커스도 받지 않는다.
 */
export function Term({ id, focusable = true, shortcut }: TermProps) {
  const label = useTermLabel(id);
  const termHelp = useUiStore((s) => s.termHelp);
  return (
    <TermTooltip id={id} shortcut={shortcut}>
      <span className={styles.term} tabIndex={focusable && termHelp ? 0 : undefined}>
        {label}
      </span>
    </TermTooltip>
  );
}
