import { useQueryClient } from "@tanstack/react-query";
import { api, type PrDetailView, type PrRowView } from "../../lib/api";
import { queryKeys, usePrDetail, usePrRepo } from "../../lib/queries";
import { Tooltip } from "../../ui/Tooltip";
import { useToastStore } from "../../ui/toastStore";
import { CommentBox } from "./CommentBox";
import { useMergeStore } from "./mergeStore";
import { PrComments } from "./PrComments";
import { Time } from "../../ui/Time";
import { useMessages } from "../../messages";
import { Button } from "../../ui/Button";
import { ErrorNotice } from "../../ui/ErrorNotice";
import { Term } from "../../ui/Term";
import { branchLine, stateLabel } from "./model";
import styles from "./PrRow.module.css";
import { Truncate } from "../../ui/Truncate";

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
        <Truncate className={styles.title}>{row.title}</Truncate>
        <span className={styles.state}>{label ?? <Term id="pr-draft" focusable={false} />}</span>
      </div>
      <div className={styles.bottom}>
        <Truncate className={styles.branches}>{branchLine(row)}</Truncate>
        {row.author && <bdi>{row.author}</bdi>}
        <Time at={row.updatedAt} now={now} />
        {row.isCurrentBranch && <span className={styles.current}>{m.pr.currentBranch}</span>}
      </div>
      {selected && <PrDetail repo={repo} number={row.number} now={now} />}
    </li>
  );
}

function PrDetail({ repo, number, now }: { repo: string; number: number; now: number }) {
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
      {data?.reviews ? (
        <p className={styles.line}>
          <Term id="review" focusable={false} /> {m.pr.reviews(data.reviews.approved, data.reviews.changesRequested)} ·{" "}
          {m.pr.comments(data.comments)}
        </p>
      ) : (
        data && <p className={styles.line}>{m.pr.comments(data.comments)}</p>
      )}
      {data?.mergeable === "conflicts" && <p className={`${styles.line} ${styles.conflicts}`}>{m.pr.conflicts}</p>}
      {data?.mergeable === "behind" && <p className={styles.line}>{m.pr.behind}</p>}
      {data?.mergeable === "blocked" && <p className={styles.line}>{m.pr.blocked}</p>}
      <div className={styles.actions}>
        {data && <PrActions repo={repo} detail={data} />}
        <Button
          variant="text"
          onClick={() => void api.openHostingPage(repo, { kind: "pullRequest", number }).catch(() => undefined)}
        >
          {m.pr.openOnGithub}
        </Button>
      </div>
      {data && <PrComments repo={repo} number={number} total={data.issueComments} now={now} />}
      {data && <CommentBox repo={repo} number={number} />}
    </div>
  );
}

/**
 * 합치기·닫기·다시 열기 단추(D-179, D-180). 권한은 저장소 정보로 정하고(작성자이거나 올릴 권한), 서버가
 * 마지막에 정한다. 합칠 수 없는 까닭은 꺼진 단추의 툴팁으로.
 */
function PrActions({ repo, detail }: { repo: string; detail: PrDetailView }) {
  const m = useMessages();
  const client = useQueryClient();
  const info = usePrRepo(repo, true).data;
  const openMerge = useMergeStore((s) => s.open);
  const show = useToastStore((s) => s.show);
  const showError = useToastStore((s) => s.showError);
  const { row } = detail;
  if (!info) return null;
  const mayClose = info.canPush || (info.viewer !== null && row.author === info.viewer);
  const refresh = () => {
    void client.invalidateQueries({ queryKey: queryKeys.prDetail(repo, row.number) });
    void client.invalidateQueries({ queryKey: ["github", "prList", repo] });
  };
  const setState = async (open: boolean) => {
    try {
      await api.executeAction(repo, { id: open ? "pr-reopen" : "pr-close", input: { number: row.number } });
      refresh();
      if (open) show(m.pr.reopen);
      else show(m.pr.closedDone, { label: m.pr.reopen, onClick: () => void setState(true) });
    } catch (e) {
      showError(e);
    }
  };
  const blocked = row.draft ? m.pr.draftNoMerge : detail.mergeable === "conflicts" ? m.pr.conflictsNoMerge : null;
  const mergeButton = (
    <Button
      variant="primary"
      disabled={blocked !== null || info.mergeMethods.length === 0}
      onClick={() => openMerge({ row, headOid: detail.headOid, mergeable: detail.mergeable, checks: detail.checks })}
    >
      <Term id="pr-merge" focusable={false} />
    </Button>
  );
  return (
    <>
      {row.state === "open" &&
        info.canMerge &&
        !info.rebaseOnly &&
        (blocked ? (
          <Tooltip content={blocked}>
            <span tabIndex={0}>{mergeButton}</span>
          </Tooltip>
        ) : (
          mergeButton
        ))}
      {row.state === "open" && info.canMerge && info.rebaseOnly && <span className={styles.muted}>{m.pr.rebaseOnly}</span>}
      {row.state === "open" && mayClose && (
        <Button variant="text" onClick={() => void setState(false)}>
          <Term id="pr-close" focusable={false} />
        </Button>
      )}
      {row.state === "closed" && mayClose && (
        <Button variant="text" onClick={() => void setState(true)}>
          {m.pr.reopen}
        </Button>
      )}
    </>
  );
}
