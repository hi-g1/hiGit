import * as RadixTooltip from "@radix-ui/react-tooltip";
import type { ReactElement, ReactNode } from "react";
import styles from "./Tooltip.module.css";

/** 호버 후 툴팁이 뜨기까지의 지연(docs/design.md) */
export const TOOLTIP_DELAY_MS = 400;

type TooltipProps = {
  content: ReactNode;
  /** 포커스 가능한 요소 하나. 키보드 포커스에도 툴팁이 뜬다 */
  children: ReactElement;
};

export function Tooltip({ content, children }: TooltipProps) {
  return (
    <RadixTooltip.Provider delayDuration={TOOLTIP_DELAY_MS}>
      <RadixTooltip.Root>
        <RadixTooltip.Trigger asChild>{children}</RadixTooltip.Trigger>
        <RadixTooltip.Portal>
          <RadixTooltip.Content className={styles.content} side="top" sideOffset={6} collisionPadding={8}>
            {content}
          </RadixTooltip.Content>
        </RadixTooltip.Portal>
      </RadixTooltip.Root>
    </RadixTooltip.Provider>
  );
}
