import { useState, type KeyboardEvent } from "react";
import { DiffView } from "../../../diff/DiffView";
import type { ChangeKind } from "../../../../lib/api";
import { listKey } from "../../../../lib/listKeys";
import { useCommitDetail } from "../../../../lib/queries";
import { Time } from "../../../../ui/Time";
import { useMessages } from "../../../../messages";
import { Button } from "../../../../ui/Button";
import { ErrorNotice } from "../../../../ui/ErrorNotice";
import type { StationInfo } from "../info";
import type { MenuCtx } from "../items";
import { DialogFrame } from "../../../../ui/DialogFrame";
import styles from "./FilesDialog.module.css";

type Props = { info: StationInfo; ctx: MenuCtx; onClose: () => void };

/** 상태 글자: 더함 +, 지움 −, 이름 바꿈 →, 그 밖 ~. */
const LETTER: Record<ChangeKind, string> = {
  added: "+",
  deleted: "−",
  renamed: "→",
  modified: "~",
  typeChanged: "~",
  conflicted: "~",
  untracked: "+",
};

/**
 * 이때 파일 보기(D-105): 그 버전에서 바뀐 파일(첫 부모와 비교)과 파일별 비교. 읽기만 한다. ↑↓로 파일을
 * 옮기고 Esc로 닫는다.
 */
export function FilesDialog({ info, ctx, onClose }: Props) {
  const m = useMessages();
  const [now] = useState(() => Date.now());
  const detail = useCommitDetail(ctx.repo, info.oid);
  const files = detail.data?.files ?? [];
  const [picked, setPicked] = useState(0);
  const at = Math.min(picked, Math.max(0, files.length - 1));
  const file = files[at];

  const onKeyDown = (e: KeyboardEvent<HTMLUListElement>) => {
    const move = listKey({ ...e, isComposing: e.nativeEvent.isComposing });
    if (move === "next" || move === "prev" || move === "first" || move === "last") {
      e.preventDefault();
      const next = { next: at + 1, prev: at - 1, first: 0, last: files.length - 1 }[move];
      setPicked(Math.max(0, Math.min(files.length - 1, next)));
    }
  };

  return (
    <DialogFrame
      title={m.filesAtVersion}
      subtitle={
        <>
          {info.shortOid} · <bdi>{info.summary}</bdi>
        </>
      }
      onClose={onClose}
      className={styles.files}
    >
      <p className={styles.meta}>
        <bdi>{info.author}</bdi> · <Time at={info.time} now={now} />
      </p>
      {detail.error && <ErrorNotice error={detail.error} />}
      {detail.data &&
        (files.length === 0 ? (
          <p className={styles.note}>{m.noFilesInVersion}</p>
        ) : (
          <div className={styles.body}>
            <div>
              {info.isMerge && <p className={styles.note}>{m.comparedBeforeMerge}</p>}
              <ul
                className={styles.list}
                role="listbox"
                aria-label={m.filesAtVersion}
                aria-activedescendant={`version-file-${at}`}
                tabIndex={0}
                autoFocus
                onKeyDown={onKeyDown}
              >
                {files.map((f, i) => (
                  <li
                    key={f.path}
                    id={`version-file-${i}`}
                    role="option"
                    aria-selected={i === at}
                    className={styles.row}
                    title={f.origPath ? m.renamedFrom(f.origPath) : undefined}
                    onClick={() => setPicked(i)}
                  >
                    <span className={styles.letter} aria-hidden="true">
                      {LETTER[f.change]}
                    </span>
                    <span className={styles.path}>{f.path}</span>
                  </li>
                ))}
              </ul>
              {detail.data.moreFiles > 0 && <p className={styles.note}>{m.moreFilesInVersion(detail.data.moreFiles)}</p>}
            </div>
            <div className={styles.diff}>
              {file && (
                <DiffView
                  repo={ctx.repo}
                  request={{ path: file.path, origPath: file.origPath, untracked: false, commit: info.oid }}
                />
              )}
            </div>
          </div>
        ))}
      <div className={styles.actions}>
        <Button variant="text" onClick={onClose}>
          {m.close}
        </Button>
      </div>
    </DialogFrame>
  );
}
