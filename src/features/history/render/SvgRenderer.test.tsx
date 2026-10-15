import { render } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { emptyScene, type Scene } from "../layout/scene";
import { SvgRenderer } from "./SvgRenderer";

function scene(): Scene {
  const s = emptyScene(200, 1000);
  s.lines.push(
    { d: "M 40 24 V 72", color: "a", width: 2, span: [24, 72] },
    { d: "M 40 900 V 960", color: "b", width: 3, span: [900, 960] },
  );
  s.stations.push(
    { id: "top", x: 40, y: 24, r: 5, stroke: 2.4, color: "a", kind: "commit", ring: 11 },
    { id: "fold", x: 40, y: 72, r: 4, stroke: 2, color: "a", kind: "fold" },
    { id: "far", x: 40, y: 940, r: 5, stroke: 2.4, color: "b", kind: "merge" },
  );
  s.texts.push({ x: 60, y: 28, text: "위", tone: "text", anchor: "start" });
  return s;
}

describe("SVG 그리기", () => {
  it("보이는 구간 밖의 선·점·글자는 그리지 않는다", () => {
    const { container } = render(
      <SvgRenderer scene={scene()} palette={(k) => `var(--graph-${k === "a" ? 1 : 2})`} viewport={{ top: 0, bottom: 200 }} />,
    );
    expect(container.querySelectorAll("path[stroke='var(--graph-2)']")).toHaveLength(0);
    expect(container.querySelectorAll("path[stroke='var(--graph-1)']").length).toBeGreaterThan(0);
    expect(container.textContent).toContain("위");
  });

  it("접힌 구간은 마름모, 지금 위치는 바깥 고리", () => {
    const { container } = render(<SvgRenderer scene={scene()} palette={() => "var(--graph-1)"} />);
    // 접힌 구간 마름모와 선 경로
    expect([...container.querySelectorAll("path")].some((p) => p.getAttribute("d")?.endsWith("Z"))).toBe(true);
    expect(container.querySelector("circle[r='11']")).not.toBeNull();
  });

  it("이름이 없으면 장식으로 숨긴다", () => {
    const { container } = render(<SvgRenderer scene={scene()} palette={() => "x"} />);
    expect(container.querySelector("svg")?.getAttribute("aria-hidden")).toBe("true");
    const named = render(<SvgRenderer scene={scene()} palette={() => "x"} label="버전 기록" />);
    expect(named.container.querySelector("svg[role='img']")?.getAttribute("aria-label")).toBe("버전 기록");
  });
});
