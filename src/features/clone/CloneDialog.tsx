import { useState } from "react";
import { api, toApiError } from "../../lib/api";
import { pickFolder } from "../../lib/dialog";
import { useAppStatus, useCloneDefaults, useGitHubStatus } from "../../lib/queries";
import { useMessages } from "../../messages";
import { Button } from "../../ui/Button";
import { Checkbox } from "../../ui/Checkbox";
import { composing, DialogFrame } from "../../ui/DialogFrame";
import { ErrorNotice } from "../../ui/ErrorNotice";
import { Term } from "../../ui/Term";
import { useOpenProject } from "../open-project/useOpenProject";
import { useProgressStore } from "../progress/progressStore";
import { CANCEL_AFTER_SECS, useProgressEvents } from "../progress/useProgress";
import { useSettingsStore } from "../settings/settingsStore";
import { useCloneStore } from "./cloneStore";
import { joinPath, suggestName } from "./name";
import { RepoList } from "./RepoList";
import styles from "./CloneDialog.module.css";
import { Truncate } from "../../ui/Truncate";

/** 복제 대화상자(D-115). 열린 동안만 그린다. */
export function CloneDialog() {
  const open = useCloneStore((s) => s.open);
  return open ? <CloneForm /> : null;
}

function CloneForm() {
  const m = useMessages();
  const close = useCloneStore((s) => s.close);
  const openSettings = useSettingsStore((s) => s.openSettings);
  const defaults = useCloneDefaults();
  const appStatus = useAppStatus();
  const github = useGitHubStatus().data;
  const project = useOpenProject();
  const [url, setUrl] = useState("");
  const [picked, setPicked] = useState<string | null>(null);
  const [name, setName] = useState("");
  const [nameEdited, setNameEdited] = useState(false);
  const [acceptOld, setAcceptOld] = useState(false);
  const [running, setRunning] = useState(false);
  const [error, setError] = useState<unknown>(null);
  const [listing, setListing] = useState(false);
  // 진행 알림은 작업 공간이 아니어도(시작 화면) 열린 동안 받는다.
  useProgressEvents();
  const progress = useProgressStore((s) => Object.values(s.byRepo).find((p) => p.action === "clone") ?? null);

  const parent = picked ?? defaults.data?.parent ?? "";
  const target = joinPath(parent, name.trim());
  const outdated = appStatus.data?.gitSecurity === "outdated";
  const ready = url.trim() !== "" && name.trim() !== "" && parent !== "" && (!outdated || acceptOld);

  const changeUrl = (value: string) => {
    setUrl(value);
    if (!nameEdited) setName(suggestName(value));
  };

  const run = async () => {
    if (!ready || running) return;
    setRunning(true);
    setError(null);
    try {
      const result = await api.executeAction(parent, {
        id: "clone",
        input: { url: url.trim(), name: name.trim(), acceptOldGit: acceptOld },
      });
      if (result.kind === "repoCreated") {
        close();
        // 프로젝트 열기 한 곳(D-032)을 거친다.
        await project.open(result.repo.path, "picked");
      }
    } catch (e) {
      setError(e);
    } finally {
      setRunning(false);
    }
  };

  const code = error === null ? null : toApiError(error).code;
  const needsConnect =
    (code === "httpAuthFailed" || code === "remoteNotFound") && github?.configured === true && !github.connected;

  const connected = github?.connected === true;
  const showList = connected && listing && !running;

  return (
    <DialogFrame title={<Term id="clone" focusable={false} />} onClose={() => !running && close()} size="wide">
      {connected && (
        <div className={styles.modes}>
          <Button variant="text" aria-pressed={!showList} disabled={running} onClick={() => setListing(false)}>
            {m.clone.url}
          </Button>
          <span aria-hidden="true">·</span>
          <Button variant="text" aria-pressed={showList} disabled={running} onClick={() => setListing(true)}>
            {m.clone.myRepos}
          </Button>
        </div>
      )}
      {showList ? (
        <RepoList
          onPick={(r) => {
            changeUrl(`https://github.com/${r.owner}/${r.name}`);
            setListing(false);
          }}
        />
      ) : (
      <label className={styles.label}>
        {connected ? null : m.clone.url}
        <input
          aria-label={connected ? m.clone.url : undefined}
          className={styles.field}
          value={url}
          placeholder={m.clone.urlPlaceholder}
          autoFocus
          spellCheck={false}
          disabled={running}
          onChange={(e) => changeUrl(e.target.value)}
          onPaste={(e) => {
            const text = e.clipboardData.getData("text").trim();
            if (text) {
              e.preventDefault();
              changeUrl(text);
            }
          }}
          onKeyDown={(e) => {
            if (e.key === "Enter" && !composing(e)) void run();
          }}
        />
      </label>
      )}
      <div className={styles.row}>
        <span className={styles.label}>{m.clone.saveIn}</span>
        <Truncate from="start" className={styles.parent}>
          {parent}
        </Truncate>
        <Button
          variant="text"
          disabled={running}
          onClick={() => {
            void pickFolder().then((path) => path && setPicked(path));
          }}
        >
          {m.clone.change}
        </Button>
      </div>
      <label className={styles.label}>
        {m.clone.folderName}
        <input
          className={styles.field}
          value={name}
          spellCheck={false}
          disabled={running}
          onChange={(e) => {
            setName(e.target.value);
            setNameEdited(true);
          }}
          onKeyDown={(e) => {
            if (e.key === "Enter" && !composing(e)) void run();
          }}
        />
      </label>
      {name.trim() !== "" && parent !== "" && (
        <Truncate from="start" className={styles.target}>
          {target}
        </Truncate>
      )}
      {outdated && (
        <label className={styles.check}>
          <Checkbox checked={acceptOld} disabled={running} onChange={(e) => setAcceptOld(e.target.checked)} />
          {m.clone.oldGit}
        </label>
      )}
      {error !== null && (
        <ErrorNotice error={error}>
          {needsConnect && (
            <Button
              variant="secondary"
              onClick={() => {
                close();
                openSettings(true);
              }}
            >
              {m.settings.connect}
            </Button>
          )}
        </ErrorNotice>
      )}
      <div className={styles.actions}>
        {running && (
          <span className={styles.progress} role="status">
            {m.clone.running(progress?.elapsedSecs ?? 0)}
          </span>
        )}
        {running && (progress?.elapsedSecs ?? 0) >= CANCEL_AFTER_SECS && (
          <Button variant="text" onClick={() => void api.cancelAction(target).catch(() => undefined)}>
            {m.clone.cancel}
          </Button>
        )}
        {!running && (
          <Button variant="text" onClick={close}>
            {m.clone.close}
          </Button>
        )}
        <Button variant="primary" disabled={!ready} busy={running} onClick={() => void run()}>
          {m.clone.run}
        </Button>
      </div>
    </DialogFrame>
  );
}
