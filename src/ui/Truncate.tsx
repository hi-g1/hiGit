import { useCallback, useEffect, useRef, useState, type ReactElement, type ReactNode } from "react";
import { Tooltip } from "./Tooltip";
import styles from "./Truncate.module.css";

type TruncateProps = {
  children: ReactNode;
  /** "start"는 앞을 줄여 끝(파일에 가까운 폴더)이 보이게 한다. 경로에 쓴다. */
  from?: "end" | "start";
  /** 툴팁에 보일 전체 글. children이 글이 아니면 꼭 준다. */
  full?: string;
  /** false면 툴팁 없이 자르기만 한다. 여러 줄을 `TruncateGroup` 하나로 묶을 때. */
  tip?: boolean;
  className?: string;
};

/**
 * 한 줄에 넘치는 글을 말줄임으로 자른다. 잘렸을 때만 마우스를 올리거나
 * 감싼 단추에 포커스가 가면 전체를 툴팁으로 보인다(C14). `title`은 쓰지 않는다.
 */
export function Truncate({ children, from = "end", full, tip = true, className }: TruncateProps) {
  const cls = [styles.text, from === "start" ? styles.start : "", className ?? ""].filter(Boolean).join(" ");
  // 이름은 사용자가 쓴 글이라 bdi로 둘레 글과 방향을 나눈다. 앞 말줄임은 바깥이
  // 오른쪽부터 채우고 글은 bdi 안에서 왼쪽→오른쪽 그대로다.
  const text = (
    <span className={cls} data-truncate="">
      {typeof children !== "string" ? children : from === "start" ? <bdi dir="ltr">{children}</bdi> : <bdi>{children}</bdi>}
    </span>
  );
  if (!tip) return text;
  return <TruncateGroup full={full ?? (typeof children === "string" ? children : "")}>{text}</TruncateGroup>;
}

/** 안쪽 `Truncate` 가운데 하나라도 잘렸으면 `full`을 툴팁으로 보인다. */
export function TruncateGroup({ full, children }: { full: string; children: ReactElement }) {
  const ref = useRef<HTMLElement | null>(null);
  const [open, setOpen] = useState(false);
  const cut = useCallback(() => {
    const el = ref.current;
    if (!el) return false;
    const parts = el.matches("[data-truncate]") ? [el] : [...el.querySelectorAll<HTMLElement>("[data-truncate]")];
    return parts.some((p) => p.scrollWidth > p.clientWidth);
  }, []);

  // 글 자체는 포커스를 받지 않으므로 감싼 단추의 키보드 포커스를 따른다.
  // 목록(role이 있는 것)은 고른 행을 따로 알리므로 따르지 않는다.
  useEffect(() => {
    const host = ref.current?.closest<HTMLElement>("button, a[href], [tabindex]:not([role])");
    if (!host) return;
    let pointer = false;
    const down = () => (pointer = true);
    const show = () => {
      if (!pointer) setOpen(cut());
      pointer = false;
    };
    const hide = () => setOpen(false);
    host.addEventListener("pointerdown", down);
    host.addEventListener("focus", show);
    host.addEventListener("blur", hide);
    return () => {
      host.removeEventListener("pointerdown", down);
      host.removeEventListener("focus", show);
      host.removeEventListener("blur", hide);
    };
  }, [cut]);

  return (
    <Tooltip
      content={full}
      open={open}
      onOpenChange={(next) => setOpen(next && cut())}
      triggerRef={(el) => {
        ref.current = el;
      }}
    >
      {children}
    </Tooltip>
  );
}
