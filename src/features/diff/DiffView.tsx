import { useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import type { DiffRequest } from "../../lib/api";
import { DEFAULT_SETTINGS } from "../../lib/display";
import { useAppSettings, useDiff, useDiffView } from "../../lib/queries";
import { useDelayedFlag } from "../../lib/useDelayed";
import { changeSettings } from "../../lib/settingsSave";
import { useMessages } from "../../messages";
import { EmptyState } from "../../ui/EmptyState";
import { Loading } from "../../ui/Loading";
import type { FileDiffView } from "../../lib/api";
import { ErrorNotice } from "../../ui/ErrorNotice";
import { TermTooltip } from "../../ui/Term";
import styles from "./DiffView.module.css";
import { countChanges, toDiffRows } from "./model";

/**
 * 고른 파일을 마지막 버전과 한 줄로 비교한다(나란히 보기는 나중, 002). 생략된 변경 없는 줄은 "변경 없는 N줄 ·
 * 펼치기"로 보이고 누르면 전체 문맥을 한 번 읽어 그 구간을 펼친다(D-156). "줄 바꿈"은 기본 켬(D-144).
 * 다른 파일을 고르면 새 비교가 올 때까지 이전 파일의 머리와 줄을 그대로 두고(D-193), 300ms가 넘으면 머리 오른쪽에
 * "불러오는 중…"을 보인다. 처음 로드는 빈 줄 모양. 펼친 상태는 보이는 파일이 바뀔 때 버린다(안쪽 `key`).
 */
export function DiffView({ repo, request }: { repo: string; request: DiffRequest }) {
  const m = useMessages();
  const client = useQueryClient();
  const diff = useDiffView(repo, request);
  const shown = diff.data;
  const late = useDelayedFlag(diff.isPlaceholderData && diff.isFetching);
  const wrap = (useAppSettings().data ?? DEFAULT_SETTINGS).diffWrap;
  const view = shown?.view;
  const showable = view && view.link === null && !view.binary;
  const shownPath = shown?.request.path ?? request.path;

  return (
    <div className={styles.view}>
      {/* 파일 이름과 숫자만(D-049, r-home.png). 비교 보기라는 이름은 칸의 접근성 이름과 호버로 남긴다. */}
      <header className={styles.head}>
        <TermTooltip id="diff">
          <h2 className={styles.path}>
            {shownPath}
          </h2>
        </TermTooltip>
        {showable && (
          <button
            type="button"
            className={styles.wrapToggle}
            aria-pressed={wrap}
            onClick={() => changeSettings(client, { diffWrap: !wrap })}
          >
            {m.wrapLines}
          </button>
        )}
        {showable && <Summary {...countChanges(view)} />}
        {late && (
          <span className={styles.loading} role="status">
            {m.loading}
          </span>
        )}
      </header>
      <div className={styles.body}>
        <Loading pending={diff.isPending} skeleton={6} />
        {diff.error && <ErrorNotice error={diff.error} />}
        {shown?.view && (
          <DiffBody
            key={`${shown.request.path}\u0000${shown.request.commit ?? ""}`}
            repo={repo}
            request={shown.request}
            view={shown.view}
            wrap={wrap}
          />
        )}
      </div>
    </div>
  );
}

/** 보이는 파일의 줄들. 펼친 구간은 이 파일 동안만 남는다. */
function DiffBody({ repo, request, view, wrap }: { repo: string; request: DiffRequest; view: FileDiffView; wrap: boolean }) {
  const m = useMessages();
  const [expanded, setExpanded] = useState<ReadonlySet<string>>(new Set());
  const full = useDiff(repo, { ...request, fullContext: true }, { enabled: expanded.size > 0 });
  if (view.link !== null) return <EmptyState>{m.linkTarget(view.link)}</EmptyState>;
  if (view.binary) return <EmptyState>{m.binaryFile}</EmptyState>;
  if (view.hunks.length === 0) return <EmptyState>{m.noDiff}</EmptyState>;
  return (
    <>
      {full.error && <ErrorNotice error={full.error} />}
      <table className={`${styles.table} ${wrap ? styles.wrapOn : ""}`}>
        <tbody>
          {toDiffRows(view, full.data, expanded).map((row) =>
            row.type === "fold" ? (
              <tr key={row.key} className={styles.fold}>
                <td colSpan={4}>
                  {row.unavailable ? (
                    <span>{m.diffTruncated}</span>
                  ) : (
                    <button
                      type="button"
                      className={styles.foldButton}
                      disabled={expanded.has(row.key)}
                      onClick={() => setExpanded((prev) => new Set(prev).add(row.key))}
                    >
                      {m.diffFold(row.count)}
                    </button>
                  )}
                  {row.section && <span className={styles.section}>{row.section}</span>}
                </td>
              </tr>
            ) : (
              <tr key={row.key} className={styles[row.kind]}>
                <td className={styles.no}>{row.oldLine}</td>
                <td className={styles.no}>{row.newLine}</td>
                <td className={styles.sign} aria-hidden="true">
                  {row.sign}
                </td>
                <td className={styles.code} data-selectable>
                  {row.text}
                  {row.noNewlineAtEnd && <span className={styles.eof}>{m.noNewlineAtEnd}</span>}
                </td>
              </tr>
            ),
          )}
        </tbody>
      </table>
      {view.truncated && <p className={styles.note}>{m.diffTruncated}</p>}
    </>
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
