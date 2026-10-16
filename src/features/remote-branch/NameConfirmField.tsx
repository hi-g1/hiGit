import { useBranchProtection } from "../../lib/queries";
import { useMessages } from "../../messages";
import styles from "../force-push/ForcePushDialog.module.css";
import { isBlockedEnter } from "../../lib/keys";

type Props = {
  repo: string;
  remote: string;
  branch: string;
  /** 이름 입력이 필요하면(원격 기본 브랜치로 보임) 입력칸을 보인다. */
  needsName: boolean;
  value: string;
  onChange: (value: string) => void;
};

/**
 * 원격 기본 브랜치의 이름 입력 확인과 서버 보호 규칙 안내(D-183). 비교는 화면의 힌트이고 core가 다시 비교한다.
 * Enter는 위험 버튼을 누르지 않고 한글 조합 중 Enter도 무시한다. 보호 여부는 확인 창이 열릴 때 한 번 읽는다.
 */
export function NameConfirmField({ repo, remote, branch, needsName, value, onChange }: Props) {
  const m = useMessages();
  const protection = useBranchProtection(repo, remote, branch, true).data;
  return (
    <>
      {protection === true && <p className={styles.others}>{m.nameConfirm.protectedOnServer}</p>}
      {needsName && (
        <label className={styles.counts}>
          <span>{m.nameConfirm.label(branch)}</span>
          <input
            className={styles.field}
            value={value}
            spellCheck={false}
            autoComplete="off"
            aria-label={m.nameConfirm.label(branch)}
            onChange={(e) => onChange(e.target.value)}
            onKeyDown={(e) => {
              if (isBlockedEnter(e)) e.preventDefault();
            }}
          />
        </label>
      )}
    </>
  );
}
