import * as RadixToast from "@radix-ui/react-toast";
import { useEffect, useRef } from "react";
import { useMessages } from "../messages";
import { Button } from "./Button";
import { ErrorNotice } from "./ErrorNotice";
import { Kbd } from "./Kbd";
import styles from "./Toast.module.css";
import { useToastStore, type ToastMessage } from "./toastStore";

/** 토스트가 떠 있는 시간. 마우스를 올리면 Radix가 그 토스트만 멈춘다. */
export const TOAST_DURATION_MS = 8000;

/**
 * 결과 알림과 "되돌리기"를 화면 아래 가운데에 셋까지 쌓아 띄운다(D-042, D-192). 오류는 닫을 때까지 남는다(D-166).
 * 쌓인 높이를 `--toast-clearance`로 알려 스크롤 칸이 포커스 줄을 토스트 밑에 두지 않게 한다. 앱에 하나만 둔다.
 */
export function ToastHost() {
  const toasts = useToastStore((s) => s.toasts);
  const viewport = useRef<HTMLOListElement>(null);
  useEffect(() => {
    const el = viewport.current;
    if (!el || typeof ResizeObserver === "undefined") return;
    const root = document.documentElement;
    const observer = new ResizeObserver(() => {
      const height = el.getBoundingClientRect().height;
      root.style.setProperty("--toast-clearance", height > 0 ? `${Math.ceil(height + 24)}px` : "0px");
    });
    observer.observe(el);
    return () => {
      observer.disconnect();
      root.style.removeProperty("--toast-clearance");
    };
  }, []);
  return (
    <RadixToast.Provider duration={TOAST_DURATION_MS} swipeDirection="down">
      {toasts.map((toast) => (toast.error ? <ErrorToast key={toast.id} toast={toast} /> : <PlainToast key={toast.id} toast={toast} />))}
      <RadixToast.Viewport ref={viewport} className={styles.viewport} />
    </RadixToast.Provider>
  );
}

function ErrorToast({ toast }: { toast: ToastMessage }) {
  const m = useMessages();
  const clear = useToastStore((s) => s.clear);
  return (
    <RadixToast.Root
      className={styles.errorToast}
      duration={Infinity}
      type="foreground"
      onOpenChange={(open) => {
        if (!open) clear(toast.id);
      }}
    >
      <ErrorNotice error={toast.error?.value}>
        {toast.action && (
          <Button
            variant="secondary"
            onClick={() => {
              toast.action?.onClick();
              clear(toast.id);
            }}
          >
            {toast.action.label}
          </Button>
        )}
      </ErrorNotice>
      <RadixToast.Close asChild>
        <Button variant="text" className={styles.close}>
          {m.close}
        </Button>
      </RadixToast.Close>
    </RadixToast.Root>
  );
}

function PlainToast({ toast }: { toast: ToastMessage }) {
  const clear = useToastStore((s) => s.clear);
  const action = toast.action;
  return (
    <RadixToast.Root
      className={styles.toast}
      onOpenChange={(open) => {
        if (!open) clear(toast.id);
      }}
    >
      <RadixToast.Title className={styles.text}>{toast.text}</RadixToast.Title>
      {action && (
        <RadixToast.Action asChild altText={action.kbd ? `${action.label} ${action.kbd}` : action.label}>
          <button
            type="button"
            className={styles.action}
            onClick={() => {
              action.onClick();
              clear(toast.id);
            }}
          >
            {action.label}
            {action.kbd && <Kbd>{action.kbd}</Kbd>}
          </button>
        </RadixToast.Action>
      )}
    </RadixToast.Root>
  );
}
