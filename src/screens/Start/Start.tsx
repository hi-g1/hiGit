import { useQueryClient } from "@tanstack/react-query";
import { UndoKeyNotice } from "../../features/undo/UndoKeyNotice";
import { Palette } from "../../features/palette/Palette";
import { useToastStore } from "../../ui/toastStore";
import { useEffect, useState } from "react";
import { ActionButton } from "../../features/actions/ActionButton";
import { useCloneStore } from "../../features/clone/cloneStore";
import { useOpenFolder } from "../../features/open-project/useOpenFolder";
import { useSettingsStore } from "../../features/settings/settingsStore";
import { api, type RecentProject } from "../../lib/api";
import { queryKeys, useAppStatus, useIdentity, useProjectSummary, useRecentProjects } from "../../lib/queries";
import { SHORTCUTS } from "../../lib/shortcuts";
import { useMessages } from "../../messages";
import { useRepoStore } from "../../state/repoStore";
import { ErrorNotice } from "../../ui/ErrorNotice";
import { Kbd } from "../../ui/Kbd";
import { Logo } from "../../ui/Logo";
import { Term } from "../../ui/Term";
import { FirstRun } from "./FirstRun";
import { LastProject } from "./LastProject";
import { ProjectRow } from "./ProjectRow";
import styles from "./Start.module.css";

/**
 * 시작 화면(D-031). 위: 마지막으로 하던 곳. 아래: 다른 프로젝트. 상단 바 오른쪽: 복제·폴더 열기·설정.
 * 최근 프로젝트가 없고 이름·이메일이 비었으면 위 자리에 처음 실행 패널(D-126).
 */
export function Start() {
  const m = useMessages();
  const queryClient = useQueryClient();
  const appStatus = useAppStatus();
  const recent = useRecentProjects();
  const openFolder = useOpenFolder();
  const openRepo = useRepoStore((s) => s.openRepo);
  const openSettings = useSettingsStore((s) => s.openSettings);
  const openClone = useCloneStore((s) => s.openClone);
  const pendingInit = useRepoStore((s) => s.pendingInit);
  const setPendingInit = useRepoStore((s) => s.setPendingInit);
  const startNotice = useRepoStore((s) => s.startNotice);
  const gitReady = appStatus.isSuccess;
  const show = useToastStore((s) => s.show);
  const showError = useToastStore((s) => s.showError);
  const projects = recent.data ?? [];
  // 위쪽 자리에 둘 프로젝트. 맨 앞 폴더가 사라졌거나 저장소가 아니면 다음 것으로 넘긴다.
  const [topIndex, setTopIndex] = useState(0);
  const top = projects[topIndex] ?? projects[0];
  const others = projects.filter((p) => p !== top);
  // 고정한 프로젝트는 위에 따로 묶는다(D-203).
  const pinnedOthers = others.filter((p) => p.pinned);
  const restOthers = others.filter((p) => !p.pinned);
  const refreshRecent = () => void queryClient.invalidateQueries({ queryKey: queryKeys.recent });
  const row = (p: RecentProject) => (
    <ProjectRow
      key={p.path}
      project={p}
      now={recent.dataUpdatedAt}
      onRemove={async () => {
        await api.removeRecent(p.path);
        refreshRecent();
      }}
      onPin={(pinned) => {
        void api
          .pinProject(p.path, pinned)
          .catch((e: unknown) => showError(e))
          .finally(refreshRecent);
      }}
    />
  );
  const identity = useIdentity();
  // "나중에"는 이번 실행 동안만(저장하지 않음, D-126).
  const [firstRunLater, setFirstRunLater] = useState(false);
  const firstRun =
    !pendingInit &&
    recent.isSuccess &&
    projects.length === 0 &&
    identity.isSuccess &&
    !identity.data.complete &&
    !firstRunLater;

  return (
    <div className={styles.start}>
      <Palette repo={null} onDone={show} />
      <header className={styles.bar}>
        <Logo size="start" />
        <span className={styles.barTitle}>{m.startTitle}</span>
        <span className={styles.spacer} />
        <button
          type="button"
          className={styles.textAction}
          disabled={!gitReady}
          onClick={openClone}
          aria-keyshortcuts="Control+Shift+O"
        >
          <Term id="clone" focusable={false} /> <Kbd>{SHORTCUTS.clone.label}</Kbd>
        </button>
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
        <UndoKeyNotice />
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
          ) : appStatus.data && firstRun ? (
            <FirstRun gitVersion={appStatus.data.gitVersion} onLater={() => setFirstRunLater(true)} />
          ) : (
            appStatus.data && <p className={styles.found}>✓ {m.foundGit(appStatus.data.gitVersion)}</p>
          ))}
      </section>

      {pinnedOthers.length > 0 && (
        <section className={styles.list} aria-labelledby="start-pinned">
          <h2 id="start-pinned" className={styles.listHeader}>
            {m.pinnedSection}
          </h2>
          <ul className={styles.rows}>{pinnedOthers.map(row)}</ul>
        </section>
      )}
      <section className={styles.list}>
        <h2 className={styles.listHeader}>{others.length > 0 ? m.otherProjectsHeader : m.startTitle}</h2>
        {recent.isSuccess && projects.length === 0 && <p className={styles.empty}>{m.noProjects}</p>}
        <ul className={styles.rows}>{restOthers.map(row)}</ul>
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
