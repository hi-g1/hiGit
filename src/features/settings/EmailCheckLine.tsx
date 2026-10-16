import { useEmailCheck, useGitHubStatus } from "../../lib/queries";
import { useMessages } from "../../messages";
import { Button } from "../../ui/Button";
import { useSettingsStore } from "./settingsStore";
import styles from "./SettingsDialog.module.css";

/**
 * 이름과 이메일 아래 한 줄(D-186): 전역 이메일이 연결된 GitHub 계정의 것인지. core는 결과 종류만 준다.
 * 권한이 없으면 "이메일 확인 권한 추가"가 같은 연결 패널을 이메일 권한으로 연다. 연결 전이면 줄이 없다.
 */
export function EmailCheckLine() {
  const m = useMessages();
  const connected = useGitHubStatus().data?.connected === true;
  const state = useEmailCheck(connected).data?.state;
  const openScopeConnect = useSettingsStore((s) => s.openScopeConnect);
  if (!connected || !state) return null;
  switch (state) {
    case "onAccount":
    case "noreply":
      return <p className={styles.hint}>{m.settings.emailOnAccount}</p>;
    case "notOnAccount":
      return <p className={styles.hint}>{m.settings.emailNotOnAccount}</p>;
    case "unverified":
      return <p className={styles.hint}>{m.settings.emailUnverified}</p>;
    case "noPermission":
      return (
        <div>
          <Button variant="text" onClick={() => openScopeConnect("email")}>
            {m.settings.addEmailPermission}
          </Button>
        </div>
      );
    default:
      return null;
  }
}
