import { useStageMutation } from "../../features/stage/useStageMutation";
import type { ChangedFile } from "../../lib/api";
import { useMessages } from "../../messages";
import { useRepoStore } from "../../state/repoStore";
import { Checkbox } from "../../ui/Checkbox";
import { ErrorNotice } from "../../ui/ErrorNotice";
import rowStyles from "../../ui/ListRow.module.css";
import { TermTooltip, useTermLabel } from "../../ui/Term";
import styles from "./ChangedFiles.module.css";

/** 이름 바꿈을 빼려면 원래 경로도 함께 되돌려야 한다. */
function pathsOf(file: ChangedFile): string[] {
  return file.origPath ? [file.path, file.origPath] : [file.path];
}

/**
 * 변경 사항 목록. 행 앞 체크 상자로 이번 버전에 넣기·빼기, 행을 누르면 비교할 파일로 고른다.
 * 체크 상자에는 글자가 없고, 이름과 툴팁은 용어표에서 온다.
 */
export function ChangedFiles({ folder, files }: { folder: string; files: ChangedFile[] }) {
  const m = useMessages();
  const includeLabel = useTermLabel("stage");
  const leaveOutLabel = useTermLabel("unstage");
  const selected = useRepoStore((s) => s.selectedPath);
  const select = useRepoStore((s) => s.selectPath);
  const stage = useStageMutation(folder);

  const included = files.filter((f) => f.staged).length;
  const all = included === files.length;
  const toggleAll = () =>
    stage.mutate(
      all
        ? { include: false, paths: files.flatMap(pathsOf) }
        : { include: true, paths: files.filter((f) => !f.staged).map((f) => f.path) },
    );

  return (
    <div>
      <div className={`${rowStyles.row} ${styles.head}`}>
        <TermTooltip id={all ? "unstage" : "stage"}>
          <Checkbox
            checked={all}
            indeterminate={included > 0 && !all}
            disabled={stage.isPending}
            aria-label={`${all ? leaveOutLabel : includeLabel}: ${m.allFiles}`}
            onChange={toggleAll}
          />
        </TermTooltip>
        <span className={rowStyles.muted}>{m.allFiles}</span>
      </div>
      {stage.error && <ErrorNotice error={stage.error} />}
      <ul className={rowStyles.list}>
        {files.map((file) => {
          const isSelected = selected === file.path;
          return (
            <li
              key={`${file.origPath ?? ""}→${file.path}`}
              className={`${rowStyles.row} ${isSelected ? styles.selected : ""}`}
            >
              <TermTooltip id={file.staged ? "unstage" : "stage"}>
                <Checkbox
                  checked={file.staged}
                  disabled={stage.isPending}
                  aria-label={`${file.staged ? leaveOutLabel : includeLabel}: ${file.path}`}
                  onChange={() => stage.mutate({ include: !file.staged, paths: pathsOf(file) })}
                />
              </TermTooltip>
              <button
                type="button"
                className={styles.pick}
                aria-pressed={isSelected}
                onClick={() => select(file.path)}
              >
                <span className={`${styles.kind} ${styles[file.change]}`}>{m.changeKind[file.change]}</span>
                <span className={`${rowStyles.mono} ${rowStyles.grow}`} title={file.path}>
                  {file.path}
                </span>
                {file.origPath && <span className={rowStyles.muted}>{m.renamedFrom(file.origPath)}</span>}
              </button>
            </li>
          );
        })}
      </ul>
    </div>
  );
}
