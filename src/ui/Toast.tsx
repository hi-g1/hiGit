import * as RadixToast from "@radix-ui/react-toast";
import { useMessages } from "../messages";
import { Button } from "./Button";
import { ErrorNotice } from "./ErrorNotice";
import { Kbd } from "./Kbd";
import styles from "./Toast.module.css";
import { useToastStore } from "./toastStore";

/** 토스트가 떠 있는 시간. 마우스를 올리면 Radix가 멈춘다. */
export const TOAST_DURATION_MS = 8000;

/** 결과 알림과 "되돌리기"를 화면 아래 가운데에 띄운다(D-042). 오류는 닫을 때까지 남는다(D-166). 앱에 하나만 둔다. */
export function ToastHost() {
  const m = useMessages();
  const toast = useToastStore((s) => s.toast);
  const clear = useToastStore((s) => s.clear);
  return (
    <RadixToast.Provider duration={TOAST_DURATION_MS} swipeDirection="down">
      {toast?.error && (
        <RadixToast.Root
          key={toast.id}
          className={styles.errorToast}
          duration={Infinity}
          type="foreground"
          onOpenChange={(open) => {
            if (!open) clear();
          }}
        >
          <ErrorNotice error={toast.error.value}>
            {toast.action && (
              <Button
                variant="secondary"
                onClick={() => {
                  toast.action?.onClick();
                  clear();
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
      )}
      {toast && !toast.error && (
        <RadixToast.Root
          key={toast.id}
          className={styles.toast}
          onOpenChange={(open) => {
            if (!open) clear();
          }}
        >
          <RadixToast.Title className={styles.text}>{toast.text}</RadixToast.Title>
          {toast.action && (
            <RadixToast.Action asChild altText={toast.action.kbd ? `${toast.action.label} ${toast.action.kbd}` : toast.action.label}>
              <button
                type="button"
                className={styles.action}
                onClick={() => {
                  toast.action?.onClick();
                  clear();
                }}
              >
                {toast.action.label}
                {toast.action.kbd && <Kbd>{toast.action.kbd}</Kbd>}
              </button>
            </RadixToast.Action>
          )}
        </RadixToast.Root>
      )}
      <RadixToast.Viewport className={styles.viewport} />
    </RadixToast.Provider>
  );
}
