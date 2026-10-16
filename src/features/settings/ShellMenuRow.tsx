import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { api } from "../../lib/api";
import { queryKeys } from "../../lib/queries";
import { useMessages } from "../../messages";
import { Checkbox } from "../../ui/Checkbox";
import { ErrorNotice } from "../../ui/ErrorNotice";
import styles from "./SettingsDialog.module.css";

/** 설정 "일반"의 폴더 오른쪽 클릭 메뉴 켬·끔(D-214). 실제 등록을 읽어 보이고, 이 컴퓨터에서 못 쓰면 행이 없다. */
export function ShellMenuRow() {
  const m = useMessages();
  const client = useQueryClient();
  const label = m.shellMenuLabel;
  const key = queryKeys.shellMenu(label);
  const status = useQuery({ queryKey: key, queryFn: () => api.shellMenuStatus(label) });
  const [error, setError] = useState<unknown>(null);
  const view = status.data;
  if (!view || view.kind === "unsupported") return null;
  const set = async (on: boolean) => {
    try {
      client.setQueryData(key, await api.shellMenuSet(on, label));
      setError(null);
    } catch (e) {
      setError(e);
    }
  };
  return (
    <div className={styles.stack}>
      <label className={styles.field}>
        <span className={styles.fieldName}>{m.settings.shellMenu}</span>
        <Checkbox checked={view.installed} onChange={(e) => void set(e.currentTarget.checked)} />
      </label>
      <p className={styles.hint}>{m.settings.shellMenuHint[view.kind]}</p>
      {error !== null && <ErrorNotice error={error} />}
    </div>
  );
}
