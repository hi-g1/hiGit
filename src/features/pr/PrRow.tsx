import { api, type PrRowView } from "../../lib/api";
import { usePrDetail } from "../../lib/queries";
import { formatDate } from "../../lib/time";
import { useMessages } from "../../messages";
import { useUiStore } from "../../state/uiStore";
import { Button } from "../../ui/Button";
import { ErrorNotice } from "../../ui/ErrorNotice";
import { Term } from "../../ui/Term";
import { branchLine, stateLabel } from "./model";
import styles from "./PrRow.module.css";

/** 검토 요청 줄 하나(D-112). 저장소 글자(제목·브랜치·작성자)는 React 글자로만 그린다. */
export function PrRow({
  row,
  repo,
  selected,
  now,
  onSelect,
}: {
  row: PrRowView;
  repo: string;
  selected: boolean;
  now: number;
  onSelect: () => void;
}) {
  const m = useMessages();
  const locale = useUiStore((s) => s.locale);
  const label = stateLabel(row, m);
  return (
    <li
      id={`pr-${row.number}`}
      role="option"
      aria-selected={selected}
      className={`${styles.row} ${selected ? styles.selected : ""}`}
      onClick={onSelect}
    >
      <div className={styles.top}>
        <span className={styles.number}>#{row.number}</span>
        <bdi className={styles.title}>{row.title}</bdi>
        <span className={styles.state}>{label ?? <Term id="pr-draft" focusable={false} />}</span>
      </div>
      <div className={styles.bottom}>
        <bdi className={styles.branches}>{branchLine(row)}</bdi>
        {row.author && <bdi>{row.author}</bdi>}
        <span>{formatDate(row.updatedAt, now, locale)}</span>
        {row.isCurrentBranch && <span className={styles.current}>{m.pr.currentBranch}</span>}
      </div>
      {selected && <PrDetail repo={repo} number={row.number} />}
    </li>
  );
}

function PrDetail({ repo, number }: { repo: string; number: number }) {
  const m = useMessages();
  const detail = usePrDetail(repo, number);
  const data = detail.data;
  return (
    <div className={styles.detail} onClick={(e) => e.stopPropagation()}>
      {detail.error && <ErrorNotice error={detail.error} />}
      {data?.checks && (
        <p className={styles.line}>
          <Term id="checks" focusable={false} />{" "}
          {m.pr.checks(data.checks.success, data.checks.failed, data.checks.pending)}
          {data.checks.partial && ` · ${m.pr.partial}`}
        </p>
      )}
      {data?.mergeable === "conflicts" && <p className={`${styles.line} ${styles.conflicts}`}>{m.pr.conflicts}</p>}
      {data?.mergeable === "behind" && <p className={styles.line}>{m.pr.behind}</p>}
      {data?.mergeable === "blocked" && <p className={styles.line}>{m.pr.blocked}</p>}
      <div>
        <Button
          variant="text"
          onClick={() => void api.openHostingPage(repo, { kind: "pullRequest", number }).catch(() => undefined)}
        >
          {m.pr.openOnGithub}
        </Button>
      </div>
    </div>
  );
}
