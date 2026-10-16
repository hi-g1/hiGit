import { useStageMutation } from "../../features/stage/useStageMutation";
import type { ChangedFile } from "../../lib/api";
import { splitPath } from "../../lib/path";
import { matches } from "../../lib/shortcuts";
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
  // 넣은 파일이 없으면 버전 저장이 모두 넣고 저장하므로 모두 켠 모양으로 보인다(D-142, D-157). index는 그대로.
  const virtualAll = files.length > 0 && included === 0;
  const all = included === files.length || virtualAll;
  const toggleAll = () =>
    stage.mutate(
      virtualAll
        ? { include: true, paths: files.map((f) => f.path) }
        : all
          ? { include: false, paths: files.flatMap(pathsOf) }
          : { include: true, paths: files.filter((f) => !f.staged).map((f) => f.path) },
    );
  /** 모두 켠 모양에서 하나를 끄면 나머지를 넣는다(하나뿐이면 그것을 넣어 확정한다). */
  const toggleOne = (file: ChangedFile) => {
    if (!virtualAll) {
      stage.mutate({ include: !file.staged, paths: pathsOf(file) });
      return;
    }
    const others = files.filter((f) => f.path !== file.path).map((f) => f.path);
    stage.mutate({ include: true, paths: others.length > 0 ? others : [file.path] });
  };

  // ↑↓로 비교할 파일을 옮긴다(단축키 표의 fileList 범위, D-056).
  const onKeyDown = (e: React.KeyboardEvent<HTMLDivElement>) => {
    const step = matches("nextFile", e.nativeEvent) ? 1 : matches("prevFile", e.nativeEvent) ? -1 : 0;
    if (step === 0 || files.length === 0) return;
    e.preventDefault();
    const at = files.findIndex((f) => f.path === selected);
    const next = files[Math.min(Math.max(at + step, 0), files.length - 1)];
    if (next) select(next.path);
  };

  return (
    // 버전을 저장하면 포커스가 여기로 온다(다음 Ctrl+Z가 글자 되돌리기로 가지 않게).
    <div id="changed-files" tabIndex={-1} className={styles.list} onKeyDown={onKeyDown}>
      <div className={`${rowStyles.row} ${styles.head}`}>
        <TermTooltip id={all ? "unstage" : "stage"}>
          <Checkbox
            checked={all}
            indeterminate={included > 0 && included < files.length}
            disabled={stage.isPending}
            aria-label={`${all ? leaveOutLabel : includeLabel}: ${m.changedFilesCount(files.length)}`}
            onChange={toggleAll}
          />
        </TermTooltip>
        <TermTooltip id="status">
          <span className={rowStyles.muted} tabIndex={0}>
            {m.changedFilesCount(files.length)}
          </span>
        </TermTooltip>
      </div>
      {stage.error && <ErrorNotice error={stage.error} />}
      <ul className={rowStyles.list}>
        {files.map((file) => {
          const isSelected = selected === file.path;
          const { name, dir } = splitPath(file.path);
          return (
            <li
              key={`${file.origPath ?? ""}→${file.path}`}
              className={`${rowStyles.row} ${styles.two} ${isSelected ? styles.selected : ""}`}
            >
              <TermTooltip id={file.staged || virtualAll ? "unstage" : "stage"}>
                <Checkbox
                  checked={file.staged || virtualAll}
                  disabled={stage.isPending}
                  aria-label={`${file.staged || virtualAll ? leaveOutLabel : includeLabel}: ${file.path}`}
                  onChange={() => toggleOne(file)}
                />
              </TermTooltip>
              <button
                type="button"
                className={styles.pick}
                aria-pressed={isSelected}
                onClick={() => select(file.path)}
              >
                {/* 위 파일 이름, 아래 폴더(--text-muted), 오른쪽 상태 글자(D-143, r-home.png) */}
                <span className={styles.names} title={file.path}>
                  <span className={`${rowStyles.mono} ${styles.name}`}>{name}</span>
                  {/* 바깥은 오른쪽부터 채워 앞쪽이 잘리고, 글자는 bdi로 왼쪽→오른쪽 그대로 */}
                  <span className={styles.dir}>
                    <bdi>{file.origPath ? m.renamedFrom(file.origPath) : dir}</bdi>
                  </span>
                </span>
                <span className={`${styles.kind} ${styles[file.change]}`}>{m.changeKind[file.change]}</span>
              </button>
            </li>
          );
        })}
      </ul>
    </div>
  );
}
