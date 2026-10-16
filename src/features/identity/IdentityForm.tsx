import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState, type KeyboardEvent } from "react";
import { api } from "../../lib/api";
import { queryKeys, useIdentity } from "../../lib/queries";
import { useMessages } from "../../messages";
import { Button } from "../../ui/Button";
import { composing } from "../../ui/DialogFrame";
import { ErrorNotice } from "../../ui/ErrorNotice";
import { Term } from "../../ui/Term";
import { useToastStore } from "../../ui/toastStore";
import styles from "./IdentityForm.module.css";

/**
 * 버전에 남길 이름과 이메일(D-125). 전역 Git 설정에 "저장"을 누를 때만 쓴다. 값은 core가 다시 검사하고,
 * 토스트·로그에 넣지 않는다. 다른 form 안에 넣지 않는다(Enter가 바깥 form을 보내지 않게).
 */
export function IdentityForm({ onSaved }: { onSaved?: () => void }) {
  const m = useMessages();
  const client = useQueryClient();
  const identity = useIdentity();
  const showToast = useToastStore((s) => s.show);
  const [name, setName] = useState<string | null>(null);
  const [email, setEmail] = useState<string | null>(null);
  const shownName = name ?? identity.data?.name ?? "";
  const shownEmail = email ?? identity.data?.email ?? "";
  const changed = name !== null || email !== null;

  const save = useMutation({
    mutationFn: () =>
      api.executeAction("", { id: "identity-set", input: { name: shownName, email: shownEmail } }),
    onSuccess: async () => {
      setName(null);
      setEmail(null);
      await client.invalidateQueries({ queryKey: queryKeys.identity });
      showToast(m.identity.saved);
      onSaved?.();
    },
  });
  // 한글 조합 중 Enter는 글자를 확정하는 키라 보내지 않는다.
  const onKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter" && composing(e)) e.preventDefault();
  };

  return (
    <form
      className={styles.form}
      aria-label={m.identity.title}
      onSubmit={(e) => {
        e.preventDefault();
        if (changed && !save.isPending) save.mutate();
      }}
    >
      <p className={styles.title}>
        <Term id="identity" />
      </p>
      <div className={styles.fields}>
        <label className={styles.field}>
          <span>{m.identity.name}</span>
          <input
            className={styles.input}
            value={shownName}
            autoComplete="name"
            onChange={(e) => setName(e.currentTarget.value)}
            onKeyDown={onKeyDown}
          />
        </label>
        <label className={styles.field}>
          <span>{m.identity.email}</span>
          <input
            className={styles.input}
            type="email"
            value={shownEmail}
            autoComplete="email"
            onChange={(e) => setEmail(e.currentTarget.value)}
            onKeyDown={onKeyDown}
          />
        </label>
        <Button type="submit" variant="primary" disabled={!changed || save.isPending}>
          {m.identity.save}
        </Button>
      </div>
      {save.error && <ErrorNotice error={save.error} />}
    </form>
  );
}
