// 앱을 켤 때 실행 인자의 폴더(D-213), 없으면 마지막 프로젝트를 바로 연다(D-032). 한 번만 돈다.
import { useEffect, useRef, useState } from "react";
import { useOpenLaunched } from "../features/open-project/useLaunchRequests";
import { useOpenProject } from "../features/open-project/useOpenProject";
import { api } from "../lib/api";
import { useAppStatus } from "../lib/queries";
import { useRepoStore } from "./repoStore";

/** 확인이 끝나면 true. 그 전에는 화면이 빈 바탕을 보여 시작 화면이 잠깐 비치지 않게 한다. */
export function useStartup(): boolean {
  const appStatus = useAppStatus();
  const project = useOpenProject();
  const openLaunched = useOpenLaunched();
  const setStartNotice = useRepoStore((s) => s.setStartNotice);
  const [done, setDone] = useState(false);
  const started = useRef(false);

  useEffect(() => {
    // git이 없거나 오래되었으면 다시 열기를 하지 않는다(아래 return에서 바로 끝).
    if (started.current || appStatus.isPending || appStatus.isError) return;
    started.current = true;
    void (async () => {
      try {
        const launched = await api.takeLaunchFolder();
        if (launched !== null) {
          await openLaunched(launched);
          return;
        }
        const [last] = await api.recentProjects();
        if (!last) return;
        const result = await project.open(last.path, "recent");
        if (result.kind === "missing") setStartNotice({ kind: "lastMissing", name: last.name });
        if (result.kind === "notARepo") setStartNotice({ kind: "lastNotRepo", name: last.name });
      } catch {
        // 목록을 읽지 못하면 시작 화면에서 고르게 한다.
      } finally {
        setDone(true);
      }
    })();
  }, [appStatus.isPending, appStatus.isError, project, openLaunched, setStartNotice]);

  return done || appStatus.isError;
}
