import { useEffect } from "react";
import { NameBox } from "../../features/commit/NameBox";
import { DiffView } from "../../features/diff/DiffView";
import { PendingNotice } from "../../features/pending/PendingNotice";
import { StuckNotice } from "../../features/progress/StuckNotice";
import { useProgressEvents } from "../../features/progress/useProgress";
import { CompareFooter } from "../../features/stash/CompareFooter";
import { SyncButton } from "../../features/sync/SyncButton";
import { SyncNotice } from "../../features/sync/SyncNotice";
import { useAutoFetch } from "../../features/sync/useAutoFetch";
import { useUndo } from "../../features/undo/useUndo";
import type { RepoInfo, StatusView } from "../../lib/api";
import { useBranches, useConflictState, useStatus } from "../../lib/queries";
import { useRepoWatch } from "../../lib/useRepoWatch";
import { useMessages } from "../../messages";
import { useRepoStore } from "../../state/repoStore";
import { Button } from "../../ui/Button";
import { EmptyState } from "../../ui/EmptyState";
import { ErrorNotice } from "../../ui/ErrorNotice";
import { Activity } from "../Activity/Activity";
import { Conflict } from "../Conflict/Conflict";
import { History } from "../History/History";
import { ChangedFiles } from "./ChangedFiles";
import { HomeMap } from "./HomeMap";
import { TopBar } from "./TopBar";
import styles from "./Workspace.module.css";

/**
 * 작업 공간: 상단 바(D-045) + 홈 2단 또는 버전 기록 화면(D-058).
 * 홈 왼쪽: 버전 이름 칸(D-044)과 고친 파일 목록. 오른쪽: 고른 파일의 비교.
 */
export function Workspace({ repo }: { repo: RepoInfo }) {
  const m = useMessages();
  const view = useRepoStore((s) => s.view);
  const selectedPath = useRepoStore((s) => s.selectedPath);
  const selectPath = useRepoStore((s) => s.selectPath);
  const status = useStatus(repo.path);
  const watch = useRepoWatch(repo.path);
  // Ctrl+Z·Ctrl+Shift+Z(D-042). 버전 저장 결과 토스트도 여기서 띄운다.
  const { notifyDone } = useUndo(repo.path);
  const hasRemote = (useBranches(repo.path).data?.remotes.length ?? 0) > 0;
  // 원격 확인은 저절로(D-045, D-055). 원격이 없으면 하지 않는다.
  useAutoFetch(repo.path, hasRemote);
  // 오래 걸리는 작업의 진행(D-080).
  useProgressEvents();
  // 합치는 중이면 홈·기록 대신 충돌 해결 화면(D-069). 앱을 켜자마자 합치는 중이어도 같다.
  const conflict = useConflictState(repo.path).data;
  const merging = conflict?.active === true;
  const files = status.data?.files;
  const selected = files?.find((f) => f.path === selectedPath);
  // 홈에 들어오면 첫 파일의 비교가 열려 있다. 고른 파일이 사라지면(버전으로 저장됨) 다음 첫 파일로.
  useEffect(() => {
    if (!files) return;
    if (files.length === 0) {
      if (selectedPath !== null) selectPath(null);
    } else if (!selected) {
      selectPath(files[0]!.path);
    }
  }, [files, selected, selectedPath, selectPath]);

  return (
    <div className={styles.workspace}>
      <TopBar repo={repo} status={status.data} onDone={notifyDone} merging={merging} />
      {watch.active === false && (
        <div className={styles.watchNotice} role="status">
          <span>{m.errors.watchUnavailable}</span>
          <Button variant="secondary" onClick={watch.reload}>
            {m.reload}
          </Button>
        </div>
      )}
      <StuckNotice repo={repo.path} />
      {/* 합치는 중이면 충돌 해결 화면이 대신 알린다(D-082). */}
      {!merging && <PendingNotice repo={repo.path} otherConflicts={conflict?.otherOperation ?? false} />}
      <SyncNotice repo={repo.path} />
      {merging && conflict ? (
        <Conflict repo={repo.path} state={conflict} onDone={notifyDone} />
      ) : view === "history" ? (
        <History repo={repo.path} />
      ) : view === "activity" ? (
        <Activity repo={repo.path} onDone={notifyDone} />
      ) : (
        <>
        <HomeMap repo={repo.path} onDone={notifyDone} />
        <div className={styles.twoPane}>
          <section className={styles.left}>
            <div className={styles.scroll}>
              {status.isPending && <EmptyState>{m.loading}</EmptyState>}
              {status.error && <ErrorNotice error={status.error} />}
              {status.data && (
                <>
                  {status.data.files.length > 0 && (
                    <NameBox
                      folder={repo.path}
                      files={status.data.files}
                      onSaved={notifyDone}
                      focusAfterSave={() => document.getElementById("changed-files")?.focus()}
                    />
                  )}
                  {status.data.files.length === 0 ? (
                    <p className={styles.noFiles}>{m.noChangedFiles}</p>
                  ) : (
                    <ChangedFiles folder={repo.path} files={status.data.files} />
                  )}
                </>
              )}
            </div>
          </section>
          <section className={styles.right} aria-label={m.compareArea}>
            {selected ? (
              <>
                <DiffView
                  key={selected.path}
                  repo={repo.path}
                  request={{
                    path: selected.path,
                    origPath: selected.origPath,
                    untracked: selected.change === "untracked",
                  }}
                />
                <CompareFooter repo={repo.path} file={selected} hasChanges={(files?.length ?? 0) > 0} onDone={notifyDone} />
              </>
            ) : (
              status.data && <AllSaved repo={repo.path} status={status.data} hasRemote={hasRemote} />
            )}
          </section>
        </div>
        </>
      )}
    </div>
  );
}

/** 고친 파일이 없을 때 오른쪽(r-saved.png). 이름과 숫자만(D-049). */
function AllSaved({ repo, status, hasRemote }: { repo: string; status: StatusView; hasRemote: boolean }) {
  const m = useMessages();
  const { branch } = status;
  return (
    <div className={styles.saved}>
      {branch.name && <p className={styles.savedBranch}>{branch.name}</p>}
      <p className={styles.savedTitle}>{m.noChangedFiles}</p>
      {branch.upstream && branch.ahead > 0 && <p className={styles.savedMeta}>{m.unpushed(branch.ahead)}</p>}
      {hasRemote && branch.name && (
        <div>
          <SyncButton repo={repo} branch={branch} changedFiles={0} shortcut={false} />
        </div>
      )}
    </div>
  );
}
