import { useQueryClient } from "@tanstack/react-query";
import { ActionButton } from "../../features/actions/ActionButton";
import { useOpenFolder } from "../../features/open-project/useOpenFolder";
import { queryKeys, useAppStatus } from "../../lib/queries";
import { SHORTCUTS } from "../../lib/shortcuts";
import { useMessages } from "../../messages";
import { useRepoStore } from "../../state/repoStore";
import { Button } from "../../ui/Button";
import { ErrorNotice } from "../../ui/ErrorNotice";
import styles from "./Home.module.css";

export function Home() {
  const m = useMessages();
  const openRepo = useRepoStore((s) => s.openRepo);
  const appStatus = useAppStatus();
  const queryClient = useQueryClient();
  const open = useOpenFolder();
  // 저장소가 아닌 폴더를 골랐을 때 "버전 관리 시작"을 묻는다(Ctrl+O로 작업 공간에서 골라도).
  const plainFolder = useRepoStore((s) => s.pendingInit);
  const setPlainFolder = useRepoStore((s) => s.setPendingInit);

  const gitReady = appStatus.isSuccess;

  return (
    <main className={styles.home}>
      <div className={styles.column}>
        <h1 className={styles.title}>hiGit</h1>
        <p className={styles.tagline}>{m.appTagline}</p>

        {appStatus.error && <ErrorNotice error={appStatus.error} />}

        <div>
          <Button variant="primary" disabled={!gitReady || open.isPending} onClick={open.pick} aria-keyshortcuts="Control+O" title={SHORTCUTS.openFolder.label}>
            {m.openFolder}
          </Button>
        </div>

        {open.error && <ErrorNotice error={open.error} />}

        {plainFolder && (
          <div className={styles.notice}>
            <p>{m.notARepo}</p>
            <p className={styles.folder}>{plainFolder}</p>
            <div>
              <ActionButton
                input={{ id: "init" }}
                folder={plainFolder}
                onDone={(result) => {
                  if (result.kind !== "repoCreated") return;
                  setPlainFolder(null);
                  openRepo(result.repo);
                  void queryClient.invalidateQueries({ queryKey: queryKeys.recent });
                }}
              />
            </div>
          </div>
        )}
      </div>
    </main>
  );
}
