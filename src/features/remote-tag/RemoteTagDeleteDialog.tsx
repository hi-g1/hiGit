import { useState } from "react";
import { api } from "../../lib/api";
import { useMessages } from "../../messages";
import { ConfirmDialog } from "../../ui/ConfirmDialog";
import { ErrorNotice } from "../../ui/ErrorNotice";
import { Term } from "../../ui/Term";

type Props = {
  repo: string;
  /** 태그 이름. */
  name: string;
  /** 태그가 가리키는 버전(확인 창이 본 값). */
  oid: string;
  onDone: () => void;
  onClose: () => void;
};

/**
 * 원격 태그 지우기 확인 창(D-182, design.md "확인 대화상자"). 포커스는 "닫기"이고 Enter가 위험 버튼을 누르지
 * 않는다. 확인 토큰은 이름과 본 버전에 묶이고, 원격 태그가 내 태그와 같을 때만 core가 지운다.
 */
export function RemoteTagDeleteDialog({ repo, name, oid, onDone, onClose }: Props) {
  const m = useMessages();
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<unknown>(null);
  const confirm = async () => {
    setPending(true);
    setError(null);
    const input = { id: "remote-tag-delete", input: { name, expect: oid } } as const;
    try {
      const preview = await api.previewAction(repo, input);
      await api.executeAction(repo, input, preview.token);
      onDone();
    } catch (e) {
      setError(e);
    } finally {
      setPending(false);
    }
  };
  return (
    <ConfirmDialog
      open
      title={<Term id="remote-tag-delete" focusable={false} />}
      subtitle={<bdi>{name}</bdi>}
      dangerLabel={m.remoteTag.delete}
      closeLabel={m.close}
      pending={pending}
      onConfirm={() => void confirm()}
      onClose={() => !pending && onClose()}
    >
      <p>{m.remoteTag.body}</p>
      {error !== null && <ErrorNotice error={error} />}
    </ConfirmDialog>
  );
}
