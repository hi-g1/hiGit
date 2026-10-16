import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useState, type ReactNode } from "react";
import { Start } from "./screens/Start/Start";
import { Workspace } from "./screens/Workspace/Workspace";
import { CloneDialog } from "./features/clone/CloneDialog";
import { IdentityFix } from "./features/identity/IdentityFix";
import { useCloneStore } from "./features/clone/cloneStore";
import { TrustDialog } from "./features/open-project/TrustDialog";
import { SettingsDialog } from "./features/settings/SettingsDialog";
import { useFontShortcuts } from "./features/settings/useFontShortcuts";
import { useOpenFolder } from "./features/open-project/useOpenFolder";
import { useAppStatus } from "./lib/queries";
import { useDisplaySync } from "./lib/useDisplaySync";
import { useShortcut } from "./lib/useShortcut";
import { useRepoStore } from "./state/repoStore";
import { useStartup } from "./state/useStartup";
import { RemoteRefusedFix } from "./features/github/RemoteRefusedFix";
import { WorkflowFix } from "./features/github/WorkflowFix";
import { ErrorActionsContext, type ErrorActions } from "./ui/errorActions";
import { TextMenu } from "./ui/TextMenu";
import { ToastHost } from "./ui/Toast";

export function createQueryClient() {
  return new QueryClient({
    defaultOptions: {
      // git 오류는 다시 시도해도 같다.
      queries: { retry: false },
      mutations: { retry: false },
    },
  });
}

/** 오류 줄마다 붙는 공용 해결 단추(D-126). */
const ERROR_ACTIONS: ErrorActions = {
  identityMissing: <IdentityFix />,
  privateEmailBlocked: <IdentityFix />,
  githubWorkflowScope: <WorkflowFix />,
  remoteRefused: <RemoteRefusedFix />,
};

export function AppProviders({ children, client }: { children: ReactNode; client?: QueryClient }) {
  const [queryClient] = useState(() => client ?? createQueryClient());
  return (
    <QueryClientProvider client={queryClient}>
      <ErrorActionsContext.Provider value={ERROR_ACTIONS}>
        {children}
        <ToastHost />
        <TextMenu />
      </ErrorActionsContext.Provider>
    </QueryClientProvider>
  );
}

/** 라우터 없이 열린 저장소가 있으면 작업 공간, 없으면 홈. `client`는 시작 때 설정을 미리 읽은 것(main.tsx). */
export function App({ client }: { client?: QueryClient }) {
  return (
    <AppProviders client={client}>
      <DisplaySync />
      <Screens />
    </AppProviders>
  );
}

/** 글자 크기·OS 배율을 화면에 적용하고 글자 크기 단축키를 받는다(D-146, D-152). */
function DisplaySync() {
  useDisplaySync();
  useFontShortcuts();
  return null;
}

function Screens() {
  const repo = useRepoStore((s) => s.repo);
  const gitReady = useAppStatus().isSuccess;
  const openFolder = useOpenFolder();
  // Ctrl+O: 시작 화면과 작업 공간 어디서나 폴더 열기(D-033)
  useShortcut("openFolder", openFolder.pick, gitReady);
  // Ctrl+Shift+O: 어디서나 복제(D-115)
  const openClone = useCloneStore((s) => s.openClone);
  useShortcut("clone", openClone, gitReady);
  // 마지막 프로젝트를 여는 동안은 빈 바탕(시작 화면이 잠깐 보였다 사라지지 않게)
  const ready = useStartup();
  if (!ready) return null;
  return (
    <>
      {repo ? <Workspace repo={repo} /> : <Start />}
      <TrustDialog />
      <SettingsDialog />
      <CloneDialog />
    </>
  );
}
