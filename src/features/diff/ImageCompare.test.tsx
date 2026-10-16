import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";
import type { ImageBlob } from "../../lib/api";
import { ImageCompare } from "./ImageCompare";

const blob = (mime: ImageBlob["mime"], data = "AAAA"): ImageBlob => ({ mime, data, bytes: 3 });

describe("이미지 비교(D-206)", () => {
  it("처음은 나란히, 두 장 모두 data 주소의 img", () => {
    render(<ImageCompare name="logo.png" before={blob("image/png")} after={blob("image/png", "BBBB")} />);
    const before = screen.getByRole("img", { name: "logo.png · 이전" });
    const after = screen.getByRole("img", { name: "logo.png · 지금" });
    expect(before).toHaveAttribute("src", "data:image/png;base64,AAAA");
    expect(after).toHaveAttribute("src", "data:image/png;base64,BBBB");
    expect(screen.getByRole("radio", { name: "나란히" })).toHaveAttribute("aria-checked", "true");
  });

  it("겹쳐 보기와 밀어 보기의 슬라이더", async () => {
    render(<ImageCompare name="logo.png" before={blob("image/png")} after={blob("image/png")} />);
    await userEvent.click(screen.getByRole("radio", { name: "겹쳐 보기" }));
    const opacity = screen.getByRole("slider", { name: "지금 쪽 진하기" });
    expect(opacity).toHaveValue("50");
    fireEvent.change(opacity, { target: { value: "80" } });
    expect(opacity).toHaveValue("80");
    // 방향키로 고르기(radiogroup)
    await userEvent.keyboard("{ArrowRight}");
    expect(screen.getByRole("radio", { name: "밀어 보기" })).toHaveAttribute("aria-checked", "true");
    expect(screen.getByRole("slider", { name: "가르는 위치" })).toHaveValue("50");
    expect(screen.queryByRole("slider", { name: "지금 쪽 진하기" })).not.toBeInTheDocument();
  });

  it("한쪽만 있으면 한 장과 새 파일·삭제", () => {
    const { unmount } = render(<ImageCompare name="new.gif" before={null} after={blob("image/gif")} />);
    expect(screen.getAllByRole("img")).toHaveLength(1);
    expect(screen.getByText(/새 파일/)).toBeInTheDocument();
    expect(screen.queryByRole("radiogroup")).not.toBeInTheDocument();
    unmount();
    render(<ImageCompare name="old.gif" before={blob("image/gif")} after={null} />);
    expect(screen.getByText(/삭제/)).toBeInTheDocument();
  });

  it("svg도 img로만 그리고 DOM에 svg 요소를 넣지 않는다", () => {
    const { container } = render(
      <ImageCompare name="icon.svg" before={blob("image/svg+xml", "PHN2Zz4=")} after={blob("image/svg+xml", "PHN2Zy8+")} />,
    );
    expect(screen.getByRole("img", { name: "icon.svg · 지금" })).toHaveAttribute("src", "data:image/svg+xml;base64,PHN2Zy8+");
    expect(container.querySelector("svg")).toBeNull();
  });

  it("원본 크기 단추", async () => {
    render(<ImageCompare name="big.png" before={blob("image/png")} after={blob("image/png")} />);
    const toggle = screen.getByRole("button", { name: "원본 크기" });
    expect(toggle).toHaveAttribute("aria-pressed", "false");
    await userEvent.click(toggle);
    expect(toggle).toHaveAttribute("aria-pressed", "true");
  });
});
