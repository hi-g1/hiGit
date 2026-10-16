import { useEffect } from "react";
import { useGithubRepos } from "../../lib/queries";

/** 만들 곳 후보를 찾는 저장소 목록 쪽 수(D-184, 새 권한 없이 목록의 조직 소유자에서). */
const OWNER_PAGES = 3;

/**
 * "GitHub에 올리기"의 만들 곳 후보 조직(D-184). 내 저장소 목록 첫 3쪽에서 소유자가 조직인 것만 중복 없이
 * 이름순으로 준다. 대화상자가 열린 동안만 읽는다.
 */
export function useOwners(enabled: boolean): string[] {
  const repos = useGithubRepos(enabled);
  const pages = repos.data?.pages ?? [];
  const { hasNextPage, isFetchingNextPage, fetchNextPage } = repos;
  useEffect(() => {
    if (enabled && hasNextPage && !isFetchingNextPage && pages.length < OWNER_PAGES) void fetchNextPage();
  }, [enabled, hasNextPage, isFetchingNextPage, pages.length, fetchNextPage]);
  const orgs = new Map<string, string>();
  for (const row of pages.flatMap((p) => p.items)) {
    if (row.ownerIsOrg && !orgs.has(row.owner.toLowerCase())) orgs.set(row.owner.toLowerCase(), row.owner);
  }
  return [...orgs.values()].sort((a, b) => a.localeCompare(b));
}
