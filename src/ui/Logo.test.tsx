import { render } from "@testing-library/react";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { Logo } from "./Logo";

describe("로고(D-163)", () => {
  it("브랜드 SVG 두 장(라이트·다크)을 그대로 쓴다", () => {
    const { container } = render(<Logo size="start" />);
    const sources = [...container.querySelectorAll("img")].map((img) => img.getAttribute("src") ?? "");
    expect(sources).toHaveLength(2);
    expect(sources[0]).toMatch(/logo-light/);
    expect(sources[1]).toMatch(/logo-dark/);
    for (const img of container.querySelectorAll("img")) expect(img).toHaveAttribute("alt", "hiGit");
  });

  it("다크 로고는 화면 모드 규칙으로만 보인다", () => {
    const css = readFileSync(join(process.cwd(), "src/ui/Logo.module.css"), "utf8");
    expect(css).toMatch(/\.logo \.dark \{\s*display: none;/);
    expect(css).toContain(':global(:root[data-theme="dark"]) .logo .dark');
    expect(css).toContain(':global(:root:not([data-theme="light"])) .logo .dark');
    // 색은 SVG 파일에만 있다.
    expect(css).not.toMatch(/#[0-9a-f]{3,6}/i);
  });
});
