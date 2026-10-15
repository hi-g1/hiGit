import { useQueryClient } from "@tanstack/react-query";
import { useCallback } from "react";
import { ApiError, api } from "../../lib/api";
import { SHORTCUTS } from "../../lib/shortcuts";
import { useShortcut } from "../../lib/useShortcut";
import { useMessages } from "../../messages";
import { useToastStore } from "../../ui/toastStore";
import { useUndoStore } from "./undoStore";

/**
 * 방금 한 일 되돌리기(Ctrl+Z)와 다시 하기(Ctrl+Shift+Z, Ctrl+Y)(D-042).
 * 입력칸에 포커스가 있으면 단축키는 글자 되돌리기로 간다(단축키 표의 inTextField).
 */
export function useUndo(repo: string) {
  const m = useMessages();
  const queryClient = useQueryClient();
  const show = useToastStore((s) => s.show);
  const setRestoredMessage = useUndoStore((s) => s.setRestoredMessage);

  const refresh = useCallback(() => {
    for (const key of [["status", repo], ["log", repo], ["branches", repo], ["summary", repo]]) {
      void queryClient.invalidateQueries({ queryKey: key });
    }
  }, [queryClient, repo]);

  const fail = useCallback(
    (e: unknown) => {
      const code = e instanceof ApiError ? e.code : "internal";
      show(m.errors[code]);
    },
    [m, show],
  );

  const redo = useCallback(async () => {
    try {
      await api.redoLast(repo);
      refresh();
      setRestoredMessage(null);
      show(m.redone);
    } catch (e) {
      fail(e);
    }
  }, [repo, refresh, setRestoredMessage, show, m, fail]);

  const undo = useCallback(async () => {
    try {
      const result = await api.undoLast(repo);
      refresh();
      if (result.message !== null) setRestoredMessage(result.message);
      show(m.undone, { label: m.redo, kbd: SHORTCUTS.redo.label, onClick: () => void redo() });
    } catch (e) {
      fail(e);
    }
  }, [repo, refresh, setRestoredMessage, show, m, fail, redo]);

  useShortcut("undo", () => void undo());
  useShortcut("redo", () => void redo());

  /** 바로 한 일의 결과를 알리고 "되돌리기"를 붙인다. */
  const notifyDone = useCallback(
    (text: string) => show(text, { label: m.undo, kbd: SHORTCUTS.undo.label, onClick: () => void undo() }),
    [show, m, undo],
  );

  return { undo, redo, notifyDone };
}
