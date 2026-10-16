import { useRef, useState, type MouseEvent } from "react";
import { useOpenFile } from "../../features/files/useOpenFile";
import { useDiscardFlow } from "../../features/stash/useDiscardFlow";
import { useStageMutation } from "../../features/stage/useStageMutation";
import type { ChangedFile } from "../../lib/api";
import { isMenuKey, listKey } from "../../lib/listKeys";
import { splitPath } from "../../lib/path";
import { matches } from "../../lib/shortcuts";
import { useMessages } from "../../messages";
import { useRepoStore } from "../../state/repoStore";
import { Checkbox } from "../../ui/Checkbox";
import { ErrorNotice } from "../../ui/ErrorNotice";
import rowStyles from "../../ui/ListRow.module.css";
import { TermTooltip, useTermLabel } from "../../ui/Term";
import styles from "./ChangedFiles.module.css";
import { FileRowMenu } from "./FileRowMenu";
import { Truncate, TruncateGroup } from "../../ui/Truncate";

/** 이름 바꿈을 빼려면 원래 경로도 함께 되돌려야 한다. */
function pathsOf(file: ChangedFile): string[] {
  return file.origPath ? [file.path, file.origPath] : [file.path];
}

/**
 * 변경 사항 목록. 행 앞 체크 상자로 이번 버전에 넣기·빼기, 행을 누르면 비교할 파일로 고른다.
 * 체크 상자에는 글자가 없고, 이름과 툴팁은 용어표에서 온다.
 */
export function ChangedFiles({
  folder,
  files,
  onDone,
}: {
  folder: string;
  files: ChangedFile[];
  /** 버리면 결과 토스트와 되돌리기(useUndo의 notifyDone). */
  onDone: (text: string) => void;
}) {
  const m = useMessages();
  const includeLabel = useTermLabel("stage");
  const leaveOutLabel = useTermLabel("unstage");
  const selected = useRepoStore((s) => s.selectedPath);
  const select = useRepoStore((s) => s.selectPath);
  const stage = useStageMutation(folder);
  const discard = useDiscardFlow(folder, onDone);
  const openFile = useOpenFile(folder);
  const [menu, setMenu] = useState<{ file: ChangedFile; x: number; y: number } | null>(null);
  /** 메뉴는 목록 안 좌표에 붙는다(누른 자리, 키로 열면 줄의 왼쪽 아래). */
  const openMenu = (file: ChangedFile, list: HTMLElement, at: { clientX: number; clientY: number }) => {
    const box = list.getBoundingClientRect();
    setMenu({ file, x: at.clientX - box.left, y: at.clientY - box.top + list.scrollTop });
  };

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

  // 줄 하나가 포커스 단위(roving tabindex, D-200). 고른 파일 줄만 Tab 정지이고, 없으면 첫 줄.
  const rows = useRef<(HTMLDivElement | null)[]>([]);
  const at = Math.max(
    files.findIndex((f) => f.path === selected),
    0,
  );
  const moveTo = (index: number) => {
    const next = files[Math.min(Math.max(index, 0), files.length - 1)];
    if (!next) return;
    select(next.path);
    rows.current[files.indexOf(next)]?.focus();
  };
  const openRowMenu = (file: ChangedFile, row: HTMLElement) => {
    const list = row.closest<HTMLElement>("#changed-files");
    const box = row.getBoundingClientRect();
    if (list) openMenu(file, list, { clientX: box.left, clientY: box.bottom });
  };
  // ↑↓·Home·End로 줄을 옮기며 고르고, Space로 넣기·빼기, 메뉴 키·Shift+F10으로 줄 메뉴(단축키 표의 fileList 범위).
  const onRowKeyDown = (e: React.KeyboardEvent<HTMLDivElement>, file: ChangedFile, index: number) => {
    const native = e.nativeEvent;
    if (isMenuKey({ ...e, isComposing: native.isComposing })) {
      e.preventDefault();
      openRowMenu(file, e.currentTarget);
      return;
    }
    const move = listKey(native);
    let target: number | null = null;
    if (matches("nextFile", native)) target = index + 1;
    else if (matches("prevFile", native)) target = index - 1;
    else if (move === "first") target = 0;
    else if (move === "last") target = files.length - 1;
    if (target !== null) {
      e.preventDefault();
      moveTo(target);
    } else if (matches("toggleFile", native)) {
      e.preventDefault();
      toggleOne(file);
    }
  };

  return (
    // 버전을 저장하면 포커스가 여기로 온다(다음 Ctrl+Z가 글자 되돌리기로 가지 않게).
    <div id="changed-files" tabIndex={-1} className={styles.list}>
      <FileRowMenu
        file={menu?.file ?? null}
        anchor={{ x: menu?.x ?? 0, y: menu?.y ?? 0 }}
        onClose={() => setMenu(null)}
        onDiscard={(file) => {
          setMenu(null);
          discard.run(file);
        }}
        onReveal={(file) => openFile.reveal(file.path)}
        onOpen={(file) => openFile.open(file.path)}
      />
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
          <span className={rowStyles.muted}>{m.changedFilesCount(files.length)}</span>
        </TermTooltip>
      </div>
      {stage.error && <ErrorNotice error={stage.error} />}
      <div role="grid" aria-label={m.changedFilesCount(files.length)} className={rowStyles.list}>
        {files.map((file, index) => {
          const isSelected = selected === file.path;
          const included = file.staged || virtualAll;
          const { name, dir } = splitPath(file.path);
          return (
            <div
              key={`${file.origPath ?? ""}→${file.path}`}
              ref={(el) => {
                rows.current[index] = el;
              }}
              role="row"
              tabIndex={index === at ? 0 : -1}
              aria-selected={isSelected}
              aria-haspopup="menu"
              aria-label={m.fileRow(file.path, m.changeKind[file.change], included)}
              className={`${rowStyles.row} ${styles.two} ${styles.fileRow} ${isSelected ? styles.selected : ""}`}
              onClick={() => select(file.path)}
              // 더블클릭은 기본 앱으로 열기(첫 클릭은 지금처럼 고르기, D-216). 체크 상자는 제외.
              onDoubleClick={(e) => {
                if ((e.target as HTMLElement).closest('input[type="checkbox"]')) return;
                openFile.open(file.path);
              }}
              onFocus={(e) => {
                // 포커스를 따라 고른다(Tab으로 들어오거나 누름).
                if (e.target === e.currentTarget && !isSelected) select(file.path);
              }}
              onKeyDown={(e) => onRowKeyDown(e, file, index)}
              onContextMenu={(e: MouseEvent<HTMLDivElement>) => {
                // 웹뷰의 기본 메뉴 대신 파일 메뉴(경로 복사·고친 것 버리기).
                e.preventDefault();
                const list = e.currentTarget.closest<HTMLElement>("#changed-files");
                if (list) openMenu(file, list, e);
              }}
            >
              <div role="gridcell" className={styles.checkCell}>
                <TermTooltip id={included ? "unstage" : "stage"}>
                  {/* 체크 상자는 마우스용이다. 키보드는 줄에서 Space(D-200). */}
                  <Checkbox
                    tabIndex={-1}
                    checked={included}
                    disabled={stage.isPending}
                    aria-label={`${included ? leaveOutLabel : includeLabel}: ${file.path}`}
                    onClick={(e) => e.stopPropagation()}
                    onChange={() => toggleOne(file)}
                  />
                </TermTooltip>
              </div>
              <div role="gridcell" className={styles.pick}>
                {/* 위 파일 이름, 아래 폴더(--text-muted), 오른쪽 상태 글자(D-143, r-home.png) */}
                {/* 이름은 끝을, 폴더는 앞을 줄인다. 어느 쪽이 잘려도 전체 경로를 툴팁으로 */}
                <TruncateGroup full={file.path}>
                  <span className={styles.names}>
                    <Truncate tip={false} className={`${rowStyles.mono} ${styles.name}`}>
                      {name}
                    </Truncate>
                    <Truncate tip={false} from={file.origPath ? "end" : "start"} className={styles.dir}>
                      {file.origPath ? m.renamedFrom(file.origPath) : dir}
                    </Truncate>
                  </span>
                </TruncateGroup>
                <span className={`${styles.kind} ${styles[file.change]}`}>{m.changeKind[file.change]}</span>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
