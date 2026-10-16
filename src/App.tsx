import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useState, type ReactNode } from "react";
import { Start } from "./screens/Start/Start";
import { Workspace } from "./screens/Workspace/Workspace";
import { TrustDialog } from "./features/open-project/TrustDialog";
import { SettingsDialog } from "./features/settings/SettingsDialog";
import { useOpenFolder } from "./features/open-project/useOpenFolder";
import { useAppStatus } from "./lib/queries";
import { useShortcut } from "./lib/useShortcut";
import { useRepoStore } from "./state/repoStore";
import { useStartup } from "./state/useStartup";
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

export function AppProviders({ children, client }: { children: ReactNode; client?: QueryClient }) {
  const [queryClient] = useState(() => client ?? createQueryClient());
  return (
    <QueryClientProvider client={queryClient}>
      {children}
      <ToastHost />
    </QueryClientProvider>
  );
}

/** 라우터 없이 열린 저장소가 있으면 작업 공간, 없으면 홈. */
export function App() {
  return (
    <AppProviders>
      <Screens />
    </AppProviders>
  );
}

function Screens() {
  const repo = useRepoStore((s) => s.repo);
  const gitReady = useAppStatus().isSuccess;
  const openFolder = useOpenFolder();
  // Ctrl+O: 시작 화면과 작업 공간 어디서나 폴더 열기(D-033)
  useShortcut("openFolder", openFolder.pick, gitReady);
  // 마지막 프로젝트를 여는 동안은 빈 바탕(시작 화면이 잠깐 보였다 사라지지 않게)
  const ready = useStartup();
  if (!ready) return null;
  return (
    <>
      {repo ? <Workspace repo={repo} /> : <Start />}
      <TrustDialog />
      <SettingsDialog />
    </>
  );
}
