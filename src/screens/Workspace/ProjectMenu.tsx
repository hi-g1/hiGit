import * as Menu from "@radix-ui/react-dropdown-menu";
import { useQueryClient } from "@tanstack/react-query";
import { useCloneStore } from "../../features/clone/cloneStore";
import { useOpenFolder } from "../../features/open-project/useOpenFolder";
import { useOpenProject } from "../../features/open-project/useOpenProject";
import { api, type RepoInfo } from "../../lib/api";
import { queryKeys, useRecentProjects } from "../../lib/queries";
import { SHORTCUTS } from "../../lib/shortcuts";
import { useMessages } from "../../messages";
import { useRepoStore } from "../../state/repoStore";
import { Kbd } from "../../ui/Kbd";
import { Term } from "../../ui/Term";
import { useToastStore } from "../../ui/toastStore";
import { Tooltip } from "../../ui/Tooltip";
import styles from "./ProjectMenu.module.css";

/** 메뉴에 보일 최근 프로젝트 수(고정 먼저). */
const RECENT_IN_MENU = 5;

/**
 * 상단 바의 프로젝트 이름 = 프로젝트 메뉴 단추(D-032, D-203). 최근 프로젝트(고정 먼저), 이 프로젝트 고정,
 * 폴더 열기·복제, 다른 프로젝트 열기(시작 화면).
 */
export function ProjectMenu({ repo }: { repo: RepoInfo }) {
  const m = useMessages();
  const queryClient = useQueryClient();
  const closeRepo = useRepoStore((s) => s.closeRepo);
  const showError = useToastStore((s) => s.showError);
  const recent = useRecentProjects().data ?? [];
  const openProject = useOpenProject();
  const openFolder = useOpenFolder();
  const openClone = useCloneStore((s) => s.openClone);
  const self = recent.find((p) => p.path === repo.path);
  const others = [...recent.filter((p) => p.path !== repo.path)]
    .sort((a, b) => Number(b.pinned) - Number(a.pinned))
    .slice(0, RECENT_IN_MENU);

  const pin = (pinned: boolean) =>
    void api
      .pinProject(repo.path, pinned)
      .catch((e: unknown) => showError(e))
      .finally(() => void queryClient.invalidateQueries({ queryKey: queryKeys.recent }));

  return (
    <Menu.Root>
      {/* 제목이 단추 안에 들지 않게 h1이 단추를 감싼다. 툴팁은 늘 폴더 경로(잘린 이름도 여기서 보인다). */}
      <h1 className={styles.heading}>
        <Tooltip content={repo.path}>
          <Menu.Trigger className={styles.trigger} aria-label={`${m.projectMenu}: ${repo.name}`}>
            <span className={styles.name}>{repo.name}</span>
            <span className={styles.caret} aria-hidden="true">
              ▾
            </span>
          </Menu.Trigger>
        </Tooltip>
      </h1>
      <Menu.Portal>
        <Menu.Content className={styles.content} align="start" sideOffset={4}>
          {others.length > 0 && (
            <Menu.Group aria-label={m.recentProjects}>
              <Menu.Label className={styles.label}>{m.recentProjects}</Menu.Label>
              {others.map((p) => (
                <Menu.Item
                  key={p.path}
                  className={styles.item}
                  onSelect={() => void openProject.open(p.path, "recent").catch((e: unknown) => showError(e))}
                >
                  <span className={styles.itemName}>{p.name}</span>
                  {p.pinned && <span className={styles.muted}>{m.pinnedSection}</span>}
                </Menu.Item>
              ))}
              <Menu.Separator className={styles.separator} />
            </Menu.Group>
          )}
          {self && (
            <Menu.Item className={styles.item} onSelect={() => pin(!self.pinned)}>
              {self.pinned ? m.unpin : m.pinProject(repo.name)}
            </Menu.Item>
          )}
          <Menu.Item className={styles.item} onSelect={() => void openFolder.pick()}>
            <span className={styles.itemName}>{m.openFolder}</span>
            <Kbd>{SHORTCUTS.openFolder.label}</Kbd>
          </Menu.Item>
          <Menu.Item className={styles.item} onSelect={openClone}>
            <span className={styles.itemName}>
              <Term id="clone" />
            </span>
            <Kbd>{SHORTCUTS.clone.label}</Kbd>
          </Menu.Item>
          <Menu.Separator className={styles.separator} />
          <Menu.Item className={styles.item} onSelect={closeRepo}>
            {m.otherProjects}
          </Menu.Item>
        </Menu.Content>
      </Menu.Portal>
    </Menu.Root>
  );
}
