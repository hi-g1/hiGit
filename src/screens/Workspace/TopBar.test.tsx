import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { StatusView } from "../../lib/api";
import { TopBar } from "./TopBar";

const repo = { path: "/home/me/내 프로젝트", name: "내 프로젝트" };
const status = (branch: Partial<StatusView["branch"]>): StatusView => ({
  branch: { name: "main", hasCommits: true, upstream: null, ahead: 0, behind: 0, ...branch },
  files: [],
});

describe("TopBar", () => {
  it("프로젝트 이름과 브랜치를 보여준다", () => {
    render(<TopBar repo={repo} status={status({})} />);
    expect(screen.getByRole("heading", { name: "내 프로젝트" })).toBeInTheDocument();
    // 프로젝트 이름은 프로젝트 메뉴 단추이고, 경로는 툴팁이다.
    expect(screen.getByRole("button", { name: "프로젝트 메뉴: 내 프로젝트" })).toHaveAttribute("title", repo.path);
    expect(screen.getByText("브랜치")).toBeInTheDocument();
    expect(screen.getByText("main")).toBeInTheDocument();
  });

  it("연결된 원격 브랜치가 있으면 올릴·가져올 버전 수", () => {
    render(<TopBar repo={repo} status={status({ upstream: "origin/main", ahead: 2, behind: 1 })} />);
    expect(screen.getByText("올리기").parentElement).toHaveTextContent("올리기 2");
    expect(screen.getByText("가져오기").parentElement).toHaveTextContent("가져오기 1");
  });

  it("올린 적이 없으면 그렇게 알려준다", () => {
    render(<TopBar repo={repo} status={status({ upstream: null })} />);
    expect(screen.getByText("아직 올린 적 없음")).toBeInTheDocument();
  });

  it("특정 버전을 보는 중이면 브랜치 이름 대신 안내", () => {
    render(<TopBar repo={repo} status={status({ name: null })} />);
    expect(screen.getByText("특정 버전을 보는 중")).toBeInTheDocument();
  });

  it("저장한 버전이 없으면 올리기 상태를 보여주지 않는다", () => {
    render(<TopBar repo={repo} status={status({ hasCommits: false })} />);
    expect(screen.queryByText("아직 올린 적 없음")).not.toBeInTheDocument();
    expect(screen.queryByText("올리기")).not.toBeInTheDocument();
  });
});
