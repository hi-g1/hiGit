import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { ActionButton } from "../../features/actions/ActionButton";
import { useOpenProject } from "../../features/open-project/useOpenProject";
import { ApiError } from "../../lib/api";
import { pickFolder } from "../../lib/dialog";
import { queryKeys, useAppStatus } from "../../lib/queries";
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
  const project = useOpenProject();
  // 저장소가 아닌 폴더를 골랐을 때 "버전 관리 시작"을 보여주려고 기억한다.
  const [plainFolder, setPlainFolder] = useState<string | null>(null);

  const open = useMutation({
    mutationFn: async () => {
      const path = await pickFolder();
      if (path === null) return;
      const result = await project.open(path, "picked");
      if (result.kind === "notARepo") setPlainFolder(result.folder);
      else setPlainFolder(null);
      if (result.kind === "missing") throw new ApiError({ code: "folderNotFound", detail: path });
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
