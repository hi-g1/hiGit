import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useState, type ReactNode } from "react";
import { Home } from "./screens/Home/Home";
import { Workspace } from "./screens/Workspace/Workspace";
import { useRepoStore } from "./state/repoStore";

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
  return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
}

/** 라우터 없이 열린 저장소가 있으면 작업 공간, 없으면 홈. */
export function App() {
  const repo = useRepoStore((s) => s.repo);
  return <AppProviders>{repo ? <Workspace repo={repo} /> : <Home />}</AppProviders>;
}
