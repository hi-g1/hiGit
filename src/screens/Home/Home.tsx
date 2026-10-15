import { useMutation } from "@tanstack/react-query";
import { useState } from "react";
import { ActionButton } from "../../features/actions/ActionButton";
import { api, type FolderInfo } from "../../lib/api";
import { pickFolder } from "../../lib/dialog";
import { useAppStatus } from "../../lib/queries";
import { useMessages } from "../../messages";
import { useRepoStore } from "../../state/repoStore";
import { Button } from "../../ui/Button";
import { ErrorNotice } from "../../ui/ErrorNotice";
import styles from "./Home.module.css";

export function Home() {
  const m = useMessages();
  const openRepo = useRepoStore((s) => s.openRepo);
  const appStatus = useAppStatus();
  // 저장소가 아닌 폴더를 골랐을 때 "관리 시작"을 보여주려고 기억한다.
  const [plainFolder, setPlainFolder] = useState<FolderInfo | null>(null);

  const open = useMutation({
    mutationFn: async () => {
      const path = await pickFolder();
      return path === null ? null : api.openFolder(path);
    },
    onSuccess: (info) => {
      if (!info) return;
      if (info.repo) {
        setPlainFolder(null);
        openRepo(info.repo);
      } else {
        setPlainFolder(info);
      }
    },
  });

  const gitReady = appStatus.isSuccess;

  return (
    <main className={styles.home}>
      <div className={styles.column}>
        <h1 className={styles.title}>hiGit</h1>
        <p className={styles.tagline}>{m.appTagline}</p>

        {appStatus.error && <ErrorNotice error={appStatus.error} />}

        <div>
          <Button variant="primary" disabled={!gitReady || open.isPending} onClick={() => open.mutate()}>
            {m.openFolder}
          </Button>
        </div>

        {open.error && <ErrorNotice error={open.error} />}

        {plainFolder && (
          <div className={styles.notice}>
            <p>{m.notARepo}</p>
            <p className={styles.folder}>{plainFolder.folder}</p>
            <div>
              <ActionButton
                actionId="init"
                folder={plainFolder.folder}
                onDone={(result) => {
                  setPlainFolder(null);
                  openRepo(result.repo);
                }}
              />
            </div>
          </div>
        )}
      </div>
    </main>
  );
}
