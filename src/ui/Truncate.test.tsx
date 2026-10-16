import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { afterEach, describe, expect, it, vi } from "vitest";
import { Truncate, TruncateGroup } from "./Truncate";

/** 글 폭을 정한다. 넓으면 잘린 것으로 본다. */
function widths(scroll: number, client = 100) {
  vi.spyOn(HTMLElement.prototype, "scrollWidth", "get").mockReturnValue(scroll);
  vi.spyOn(HTMLElement.prototype, "clientWidth", "get").mockReturnValue(client);
}

afterEach(() => vi.restoreAllMocks());

describe("Truncate", () => {
  it("잘리지 않으면 툴팁이 없다", async () => {
    widths(80);
    const user = userEvent.setup();
    render(<Truncate>feature/login</Truncate>);
    await user.hover(screen.getByText("feature/login"));
    await new Promise((r) => setTimeout(r, 500));
    expect(screen.queryByRole("tooltip")).not.toBeInTheDocument();
  });

  it("잘리면 마우스를 올렸을 때 전체를 보인다", async () => {
    widths(300);
    const user = userEvent.setup();
    render(<Truncate>feature/아주-긴-브랜치-이름</Truncate>);
    await user.hover(screen.getByText("feature/아주-긴-브랜치-이름"));
    expect(await screen.findByRole("tooltip")).toHaveTextContent("feature/아주-긴-브랜치-이름");
  });

  it("감싼 단추에 키보드 포커스가 가면 보이고, 떠나면 닫는다", async () => {
    widths(300);
    const user = userEvent.setup();
    render(
      <>
        <button type="button">
          <Truncate full="src/components/LoginForm.tsx">LoginForm.tsx</Truncate>
        </button>
        <button type="button">다음</button>
      </>,
    );
    await user.tab();
    expect(await screen.findByRole("tooltip")).toHaveTextContent("src/components/LoginForm.tsx");
    await user.tab();
    expect(screen.queryByRole("tooltip")).not.toBeInTheDocument();
  });

  it("단추를 눌러 생긴 포커스로는 뜨지 않는다", async () => {
    widths(300);
    const user = userEvent.setup();
    render(
      <button type="button">
        <Truncate>LoginForm.tsx</Truncate>
      </button>,
    );
    await user.pointer({ keys: "[MouseLeft>]", target: screen.getByRole("button") });
    expect(screen.queryByRole("tooltip")).not.toBeInTheDocument();
  });

  it("경로는 앞을 줄이고 글은 bdi로 왼쪽→오른쪽 그대로 둔다", () => {
    render(<Truncate from="start">src/화면/설정 파일.ts</Truncate>);
    const bdi = screen.getByText("src/화면/설정 파일.ts");
    expect(bdi.tagName).toBe("BDI");
    expect(bdi.parentElement?.className).toMatch(/start/);
    const css = readFileSync(join(__dirname, "Truncate.module.css"), "utf8");
    const start = css.slice(css.indexOf(".start"));
    expect(start).toMatch(/direction:\s*rtl/);
    expect(start).toMatch(/text-align:\s*left/);
    expect(css).toMatch(/text-overflow:\s*ellipsis/);
  });

  it("title 속성을 쓰지 않는다", () => {
    widths(300);
    const { container } = render(<Truncate>긴 이름</Truncate>);
    expect(container.querySelector("[title]")).toBeNull();
  });

  it("묶음은 안쪽 줄 가운데 하나만 잘려도 전체를 보인다", async () => {
    vi.spyOn(HTMLElement.prototype, "scrollWidth", "get").mockImplementation(function (this: HTMLElement) {
      return this.textContent === "src/components" ? 300 : 50;
    });
    vi.spyOn(HTMLElement.prototype, "clientWidth", "get").mockReturnValue(100);
    const user = userEvent.setup();
    render(
      <TruncateGroup full="src/components/LoginForm.tsx">
        <span>
          <Truncate tip={false}>LoginForm.tsx</Truncate>
          <Truncate tip={false} from="start">
            src/components
          </Truncate>
        </span>
      </TruncateGroup>,
    );
    await user.hover(screen.getByText("LoginForm.tsx"));
    expect(await screen.findByRole("tooltip")).toHaveTextContent("src/components/LoginForm.tsx");
  });
});
