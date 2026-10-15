import { fireEvent, render, screen } from "@testing-library/react";
import { useState } from "react";
import { describe, expect, it, vi } from "vitest";
import { AppProviders, createQueryClient } from "../../App";
import { buildRoute, ROW_H, verticalScene } from "./layout/route";
import { commitNode, type RouteNode } from "./model";
import { branchPalette } from "./render/palette";
import { RouteList } from "./RouteList";

const NOW = Date.UTC(2026, 9, 16);

function nodes(count: number): RouteNode[] {
  return Array.from({ length: count }, (_, i) =>
    commitNode(`c${i}`.padEnd(40, "0"), i === count - 1 ? [] : [`c${i + 1}`.padEnd(40, "0")], {
      summary: `버전 ${i}`,
      author: "지원",
      time: NOW / 1000 - i * 3600,
    }),
  );
}

function List({ list, onActivate = () => {} }: { list: RouteNode[]; onActivate?: (id: string) => void }) {
  const [selected, setSelected] = useState<string | null>(null);
  const scene = verticalScene(buildRoute(list));
  return (
    <AppProviders client={createQueryClient()}>
      <RouteList
        nodes={list}
        scene={scene}
        palette={branchPalette("main")}
        selectedId={selected}
        onSelect={setSelected}
        onActivate={onActivate}
        now={NOW}
        label="버전 기록"
      />
    </AppProviders>
  );
}

describe("노선도 목록", () => {
  it("1만 행이어도 DOM 행은 60개 이하", () => {
    render(<List list={nodes(10_000)} />);
    expect(screen.getAllByRole("option").length).toBeLessThanOrEqual(60);
  });

  it("↓ 두 번이면 둘째 행, Home·End", () => {
    render(<List list={nodes(100)} />);
    const list = screen.getByRole("listbox", { name: "버전 기록" });
    fireEvent.keyDown(list, { key: "ArrowDown" });
    fireEvent.keyDown(list, { key: "ArrowDown" });
    expect(list.getAttribute("aria-activedescendant")).toBe("route-row-1");
    fireEvent.keyDown(list, { key: "End" });
    expect(list.getAttribute("aria-activedescendant")).toBe("route-row-99");
    // 고른 행이 화면 밖이면 스크롤한다.
    expect(list.scrollTop).toBeGreaterThan(0);
    // 창 밖이어도 고른 행은 DOM에 있다.
    expect(document.getElementById("route-row-99")).not.toBeNull();
    fireEvent.keyDown(list, { key: "Home" });
    expect(list.getAttribute("aria-activedescendant")).toBe("route-row-0");
    expect(list.scrollTop).toBe(0);
  });

  it("한글 조합 중 방향키는 무시한다", () => {
    render(<List list={nodes(10)} />);
    const list = screen.getByRole("listbox");
    fireEvent.keyDown(list, { key: "ArrowDown", isComposing: true });
    expect(list.getAttribute("aria-activedescendant")).toBeNull();
  });

  it("Enter·Space·Shift+F10·ContextMenu로 고른 역을 연다", () => {
    const onActivate = vi.fn();
    render(<List list={nodes(5)} onActivate={onActivate} />);
    const list = screen.getByRole("listbox");
    fireEvent.keyDown(list, { key: "ArrowDown" });
    for (const key of [{ key: "Enter" }, { key: " " }, { key: "F10", shiftKey: true }, { key: "ContextMenu" }]) {
      fireEvent.keyDown(list, key);
    }
    expect(onActivate).toHaveBeenCalledTimes(4);
    expect(onActivate).toHaveBeenLastCalledWith("c0".padEnd(40, "0"));
  });

  it("누르면 장면 좌표로 행을 골라 연다", () => {
    const onActivate = vi.fn();
    render(<List list={nodes(5)} onActivate={onActivate} />);
    const list = screen.getByRole("listbox");
    list.getBoundingClientRect = () => ({ top: 0, left: 0, right: 800, bottom: 600, width: 800, height: 600, x: 0, y: 0, toJSON: () => ({}) });
    Object.defineProperty(list, "clientWidth", { value: 780, configurable: true });
    fireEvent.pointerDown(list, { button: 0, clientX: 300, clientY: ROW_H * 2 + 5 });
    expect(list.getAttribute("aria-activedescendant")).toBe("route-row-2");
    expect(onActivate).toHaveBeenCalledWith("c2".padEnd(40, "0"));
    // 스크롤 막대는 고르지 않는다.
    fireEvent.pointerDown(list, { button: 0, clientX: 790, clientY: 5 });
    expect(onActivate).toHaveBeenCalledTimes(1);
  });

  it("저장소 글자는 bdi 안에 글자로, 행 이름에 이름·작성자·표시를 담는다", () => {
    const list = [
      commitNode("a".repeat(40), ["b".repeat(40)], {
        summary: "<b>굵게</b>",
        author: "지원",
        time: NOW / 1000,
        branches: ["기능"],
        tags: ["v1"],
        isHead: true,
        unpushed: true,
      }),
      commitNode("b".repeat(40), ["c".repeat(40), "d".repeat(40)], { summary: "합침", author: "민지", time: NOW / 1000 }),
    ];
    render(<List list={list} />);
    const first = screen.getAllByRole("option")[0]!;
    expect(first.querySelector("bdi")?.textContent).toBe("<b>굵게</b>");
    expect(first.querySelector("b")).toBeNull();
    const name = first.getAttribute("aria-label")!;
    for (const part of ["<b>굵게</b>", "지원", "브랜치 기능", "태그 v1", "지금 여기", "안 올림", "1번째"]) {
      expect(name).toContain(part);
    }
    expect(screen.getAllByRole("option")[1]!.getAttribute("aria-label")).toContain("합치기");
    expect(first.textContent).toContain("안 올림");
  });

  it("접힌 구간은 '버전 N개 · 펼치기', Enter로 연다", () => {
    const onActivate = vi.fn();
    const list = [
      commitNode("h".repeat(40), ["fold"], { summary: "위" }),
      { ...commitNode("fold", ["r".repeat(40)]), kind: "fold" as const, count: 12 },
      commitNode("r".repeat(40), [], { summary: "처음" }),
    ];
    render(<List list={list} onActivate={onActivate} />);
    expect(screen.getByText("버전 12개 · 펼치기")).toBeInTheDocument();
    const box = screen.getByRole("listbox");
    fireEvent.keyDown(box, { key: "ArrowDown" });
    fireEvent.keyDown(box, { key: "ArrowDown" });
    fireEvent.keyDown(box, { key: "Enter" });
    expect(onActivate).toHaveBeenCalledWith("fold");
  });
});
