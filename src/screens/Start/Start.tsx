import { useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { ActionButton } from "../../features/actions/ActionButton";
import { useOpenFolder } from "../../features/open-project/useOpenFolder";
import { useSettingsStore } from "../../features/settings/settingsStore";
import { api, type RecentProject } from "../../lib/api";
import { queryKeys, useAppStatus, useProjectSummary, useRecentProjects } from "../../lib/queries";
import { SHORTCUTS } from "../../lib/shortcuts";
import { useMessages } from "../../messages";
import { useRepoStore } from "../../state/repoStore";
import { ErrorNotice } from "../../ui/ErrorNotice";
import { Kbd } from "../../ui/Kbd";
import { LastProject } from "./LastProject";
import { ProjectRow } from "./ProjectRow";
import styles from "./Start.module.css";

/**
 * 시작 화면(D-031). 위: 마지막으로 하던 곳. 아래: 다른 프로젝트. 상단 바 오른쪽: 폴더 열기(Ctrl+O).
 * 노선도(D-017 확정 뒤), 복제(반복 4), 처음 실행의 이름·이메일(1b 뒤)은 아직 없다.
 */
export function Start() {
  const m = useMessages();
  const queryClient = useQueryClient();
  const appStatus = useAppStatus();
  const recent = useRecentProjects();
  const openFolder = useOpenFolder();
  const openRepo = useRepoStore((s) => s.openRepo);
  const openSettings = useSettingsStore((s) => s.openSettings);
  const pendingInit = useRepoStore((s) => s.pendingInit);
  const setPendingInit = useRepoStore((s) => s.setPendingInit);
  const startNotice = useRepoStore((s) => s.startNotice);
  const gitReady = appStatus.isSuccess;
  const projects = recent.data ?? [];
  // 위쪽 자리에 둘 프로젝트. 맨 앞 폴더가 사라졌거나 저장소가 아니면 다음 것으로 넘긴다.
  const [topIndex, setTopIndex] = useState(0);
  const top = projects[topIndex] ?? projects[0];
  const others = projects.filter((p) => p !== top);

  return (
    <div className={styles.start}>
      <header className={styles.bar}>
        <span className={styles.barTitle}>{m.startTitle}</span>
        <span className={styles.spacer} />
        <button
          type="button"
          className={styles.textAction}
          disabled={!gitReady || openFolder.isPending}
          onClick={openFolder.pick}
          aria-keyshortcuts="Control+O"
        >
          {m.openFolder}{" "}
          <Kbd>{SHORTCUTS.openFolder.label}</Kbd>
        </button>
        <button type="button" className={styles.textAction} onClick={() => openSettings()}>
          {m.settings.title}
        </button>
      </header>

      <section className={styles.top}>
        {appStatus.data?.gitSecurity === "outdated" && (
          <p className={styles.notice} role="status">
            {m.gitUpdate}{" "}
            <button type="button" className={styles.textAction} onClick={() => api.openGitDownload().catch(() => undefined)}>
              {m.gitDownload}
            </button>
          </p>
        )}
        {startNotice && (
          <p className={styles.notice} role="status">
            {startNotice.kind === "lastMissing" ? m.lastMissing(startNotice.name) : m.lastNotRepo(startNotice.name)}
          </p>
        )}
        {appStatus.error && <ErrorNotice error={appStatus.error} />}
        {openFolder.error && <ErrorNotice error={openFolder.error} />}
        {pendingInit && (
          <div className={styles.initBox}>
            <p>{m.notARepo}</p>
            <p className={styles.path}>{pendingInit}</p>
            <div>
              <ActionButton
                input={{ id: "init" }}
                folder={pendingInit}
                variant="primary"
                onDone={(result) => {
                  if (result.kind !== "repoCreated") return;
                  setPendingInit(null);
                  openRepo(result.repo);
                  void queryClient.invalidateQueries({ queryKey: queryKeys.recent });
                }}
              />
            </div>
          </div>
        )}
        {!pendingInit &&
          (projects.length > 0 ? (
            top && (
              <TopProject
                key={top.path}
                project={top}
                now={recent.dataUpdatedAt}
                onUnavailable={() => {
                  if (topIndex + 1 < projects.length) setTopIndex(topIndex + 1);
                }}
              />
            )
          ) : (
            appStatus.data && <p className={styles.found}>✓ {m.foundGit(appStatus.data.gitVersion)}</p>
          ))}
      </section>

      <section className={styles.list}>
        <h2 className={styles.listHeader}>{others.length > 0 ? m.otherProjectsHeader : m.startTitle}</h2>
        {recent.isSuccess && projects.length === 0 && <p className={styles.empty}>{m.noProjects}</p>}
        <ul className={styles.rows}>
          {others.map((p) => (
            <ProjectRow
              key={p.path}
              project={p}
              now={recent.dataUpdatedAt}
              onRemove={async () => {
                await api.removeRecent(p.path);
                void queryClient.invalidateQueries({ queryKey: queryKeys.recent });
              }}
            />
          ))}
        </ul>
      </section>
    </div>
  );
}

/** 위쪽 "마지막으로 하던 곳". 그 폴더를 열 수 없으면 알려서 다음 프로젝트로 넘긴다. */
function TopProject({
  project,
  now,
  onUnavailable,
}: {
  project: RecentProject;
  now: number;
  onUnavailable: () => void;
}) {
  const summary = useProjectSummary(project.path);
  const unavailable = summary.isError;
  useEffect(() => {
    if (unavailable) onUnavailable();
  }, [unavailable, onUnavailable]);
  return <LastProject project={project} summary={summary} now={now} />;
}
