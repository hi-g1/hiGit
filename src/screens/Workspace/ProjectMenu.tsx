import * as Menu from "@radix-ui/react-dropdown-menu";
import type { RepoInfo } from "../../lib/api";
import { useMessages } from "../../messages";
import { useRepoStore } from "../../state/repoStore";
import { Tooltip } from "../../ui/Tooltip";
import styles from "./ProjectMenu.module.css";

/** 상단 바의 프로젝트 이름 = 프로젝트 메뉴 단추(D-032). "다른 프로젝트 열기"로 시작 화면에 간다. */
export function ProjectMenu({ repo }: { repo: RepoInfo }) {
  const m = useMessages();
  const closeRepo = useRepoStore((s) => s.closeRepo);
  return (
    <Menu.Root>
      {/* 툴팁은 늘 폴더 경로를 보인다. 경로 끝이 이름이라 잘린 이름도 여기서 보인다. */}
      <Tooltip content={repo.path}>
        <Menu.Trigger className={styles.trigger} aria-label={`${m.projectMenu}: ${repo.name}`}>
          <h1 className={styles.name}>{repo.name}</h1>
          <span className={styles.caret} aria-hidden="true">
            ▾
          </span>
        </Menu.Trigger>
      </Tooltip>
      <Menu.Portal>
        <Menu.Content className={styles.content} align="start" sideOffset={4}>
          <Menu.Item className={styles.item} onSelect={closeRepo}>
            {m.otherProjects}
          </Menu.Item>
        </Menu.Content>
      </Menu.Portal>
    </Menu.Root>
  );
}
