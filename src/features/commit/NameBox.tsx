import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef, useState, type KeyboardEvent } from "react";
import type { ChangedFile } from "../../lib/api";
import { api } from "../../lib/api";
import { queryKeys } from "../../lib/queries";
import { SHORTCUTS, matches } from "../../lib/shortcuts";
import { useMessages } from "../../messages";
import { Button } from "../../ui/Button";
import { ErrorNotice } from "../../ui/ErrorNotice";
import { Kbd } from "../../ui/Kbd";
import { Term } from "../../ui/Term";
import { useUndoStore } from "../undo/undoStore";
import styles from "./NameBox.module.css";
import { suggestName } from "./suggest";

/** 이름과 설명을 버전 메시지로. 설명이 있으면 빈 줄을 두고 붙인다. */
export function composeMessage(name: string, description: string, suggestion: string): string {
  const title = name.trim() || suggestion;
  const body = description.trim();
  return body ? `${title}\n\n${body}` : title;
}

/** 버전 메시지를 이름과 설명으로 나눈다(되돌린 저장을 칸에 다시 채울 때). */
export function splitMessage(message: string): { name: string; description: string } {
  const [first = "", ...rest] = message.split("\n");
  return { name: first, description: rest.join("\n").trim() };
}

type Props = {
  folder: string;
  files: ChangedFile[];
  /** 저장하면 결과 토스트를 띄운다(되돌리기 포함, useUndo의 notifyDone). */
  onSaved: (text: string) => void;
  /** 저장한 뒤 포커스를 옮길 곳(파일 목록). 글상자에 남으면 다음 Ctrl+Z가 글자 되돌리기로 간다. */
  focusAfterSave?: () => void;
};

/**
 * 버전 이름(D-044). 고친 파일 목록 위에 문서 제목처럼 큰 이름 칸과 설명 칸을 둔다.
 * 비워 두면 제안 이름으로 저장하고, 넣은 파일이 없으면 고친 파일을 모두 넣고 저장한다(D-062).
 */
export function NameBox({ folder, files, onSaved, focusAfterSave }: Props) {
  const m = useMessages();
  const queryClient = useQueryClient();
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const nameRef = useRef<HTMLInputElement>(null);
  const suggestion = suggestName(files, m);
  const stageAll = !files.some((f) => f.staged);

  // 버전 저장을 되돌리면 그때 쓴 이름과 설명을 다시 채운다.
  useEffect(
    () =>
      useUndoStore.subscribe((state) => {
        if (state.restoredMessage === null) return;
        const parts = splitMessage(state.restoredMessage);
        setName(parts.name);
        setDescription(parts.description);
        state.setRestoredMessage(null);
      }),
    [],
  );

  const commit = useMutation({
    mutationFn: () =>
      api.executeAction(folder, {
        id: "commit",
        input: { message: composeMessage(name, description, suggestion), stageAll },
      }),
    onSuccess: () => {
      setName("");
      setDescription("");
      onSaved(m.saved);
      if (focusAfterSave) focusAfterSave();
      else nameRef.current?.blur();
    },
    onSettled: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.status(folder) });
      void queryClient.invalidateQueries({ queryKey: queryKeys.log(folder) });
    },
  });

  const canCommit = files.length > 0 && !commit.isPending;
  const save = () => {
    if (canCommit) commit.mutate();
  };
  // 단축키 표(D-033)로 판정한다. 한글 조합 중 Ctrl+Enter는 저장하지 않는다.
  const onKeyDown = (e: KeyboardEvent<HTMLInputElement | HTMLTextAreaElement>) => {
    if (matches("commit", e.nativeEvent)) {
      e.preventDefault();
      save();
    }
  };

  return (
    <form
      className={styles.box}
      onSubmit={(e) => {
        e.preventDefault();
        save();
      }}
    >
      <input
        ref={nameRef}
        className={styles.name}
        value={name}
        placeholder={suggestion}
        aria-label={m.versionName}
        onChange={(e) => setName(e.target.value)}
        onKeyDown={onKeyDown}
      />
      <textarea
        className={styles.description}
        value={description}
        placeholder={m.descriptionPlaceholder}
        aria-label={m.descriptionPlaceholder}
        rows={1}
        onChange={(e) => setDescription(e.target.value)}
        onKeyDown={onKeyDown}
      />
      <div className={styles.actions}>
        <Kbd>{SHORTCUTS.commit.label}</Kbd>
        <Button type="submit" variant="primary" disabled={!canCommit}>
          <Term id="commit" focusable={false} />
        </Button>
      </div>
      {files.length === 0 && <p className={styles.hint}>{m.noChangedFiles}</p>}
      {commit.error && <ErrorNotice error={commit.error} />}
    </form>
  );
}
