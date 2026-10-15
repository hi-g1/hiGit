import type { DiffRequest } from "../../lib/api";
import { useDiff } from "../../lib/queries";
import { useMessages } from "../../messages";
import { EmptyState } from "../../ui/EmptyState";
import { ErrorNotice } from "../../ui/ErrorNotice";
import { TermTooltip } from "../../ui/Term";
import styles from "./DiffView.module.css";
import { countChanges, toDiffRows } from "./model";

/** 고른 파일을 마지막 버전과 한 줄로 비교한다(나란히 보기는 나중, 002). */
export function DiffView({ repo, request }: { repo: string; request: DiffRequest }) {
  const m = useMessages();
  const diff = useDiff(repo, request);

  return (
    <div className={styles.view}>
      {/* 파일 이름과 숫자만(D-049, r-home.png). 비교 보기라는 이름은 칸의 접근성 이름과 호버로 남긴다. */}
      <header className={styles.head}>
        <TermTooltip id="diff">
          <h2 className={styles.path} title={request.path} tabIndex={0}>
            {request.path}
          </h2>
        </TermTooltip>
        {diff.data && !diff.data.binary && !diff.data.link && <Summary {...countChanges(diff.data)} />}
      </header>
      <div className={styles.body}>
        {diff.isPending && <EmptyState>{m.loading}</EmptyState>}
        {diff.error && <ErrorNotice error={diff.error} />}
        {diff.data &&
          (diff.data.link !== null ? (
            <EmptyState>{m.linkTarget(diff.data.link)}</EmptyState>
          ) : diff.data.binary ? (
            <EmptyState>{m.binaryFile}</EmptyState>
          ) : diff.data.hunks.length === 0 ? (
            <EmptyState>{m.noDiff}</EmptyState>
          ) : (
            <>
              <table className={styles.table}>
                <tbody>
                  {toDiffRows(diff.data).map((row) =>
                    row.type === "hunk" ? (
                      <tr key={row.key} className={styles.hunk}>
                        <td colSpan={4}>{row.section || "…"}</td>
                      </tr>
                    ) : (
                      <tr key={row.key} className={styles[row.kind]}>
                        <td className={styles.no}>{row.oldLine}</td>
                        <td className={styles.no}>{row.newLine}</td>
                        <td className={styles.sign} aria-hidden="true">
                          {row.sign}
                        </td>
                        <td className={styles.code}>
                          {row.text}
                          {row.noNewlineAtEnd && <span className={styles.eof}>{m.noNewlineAtEnd}</span>}
                        </td>
                      </tr>
                    ),
                  )}
                </tbody>
              </table>
              {diff.data.truncated && <p className={styles.note}>{m.diffTruncated}</p>}
            </>
          ))}
      </div>
    </div>
  );
}

function Summary({ added, removed }: { added: number; removed: number }) {
  const m = useMessages();
  return (
    <p className={styles.summary}>
      <span className={styles.sumAdded}>{m.diffAdded(added)}</span>, <span className={styles.sumRemoved}>{m.diffRemoved(removed)}</span>
    </p>
  );
}
