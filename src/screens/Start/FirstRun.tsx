import { IdentityForm } from "../../features/identity/IdentityForm";
import { useSettingsStore } from "../../features/settings/settingsStore";
import { useGitHubStatus } from "../../lib/queries";
import { useMessages } from "../../messages";
import { Button } from "../../ui/Button";
import styles from "./FirstRun.module.css";

/**
 * 처음 실행 패널(D-126, start-first.png). 이름·이메일 줄과 GitHub 연결 줄은 서로 기다리지 않는다.
 * 이름·이메일을 저장하면 시작 화면이 패널을 거두고, "나중에"는 이번 실행 동안만 숨긴다.
 */
export function FirstRun({ gitVersion, onLater }: { gitVersion: string; onLater: () => void }) {
  const m = useMessages();
  const github = useGitHubStatus().data;
  const openSettings = useSettingsStore((s) => s.openSettings);
  const connected = github?.connected === true;
  return (
    <div className={styles.panel}>
      <p className={styles.small}>{m.firstRun.once}</p>
      <h1 className={styles.title}>{m.firstRun.title}</h1>
      <ol className={styles.steps}>
        <li className={styles.step}>
          <span className={styles.mark} aria-hidden="true">
            ✓
          </span>
          <span>{m.foundGit(gitVersion)}</span>
        </li>
        <li className={styles.step}>
          <span className={`${styles.mark} ${styles.now}`} aria-hidden="true">
            ●
          </span>
          <div className={styles.body}>
            <IdentityForm />
            <p className={styles.hint}>{m.firstRun.identityHint}</p>
          </div>
        </li>
        <li className={styles.step}>
          <span className={styles.mark} aria-hidden="true">
            {connected ? "✓" : "○"}
          </span>
          <div className={styles.line}>
            <span>{m.firstRun.github}</span>
            {connected && github.login ? (
              <span className={styles.login}>@{github.login}</span>
            ) : (
              <span className={styles.hint}>{m.firstRun.githubHint}</span>
            )}
            {github?.configured && !connected && (
              <Button variant="text" onClick={() => openSettings(true)}>
                {m.firstRun.connect}
              </Button>
            )}
          </div>
        </li>
      </ol>
      <div>
        <Button variant="text" onClick={onLater}>
          {m.firstRun.later}
        </Button>
      </div>
    </div>
  );
}
