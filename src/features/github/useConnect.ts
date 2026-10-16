import { useEffect, useState } from "react";
import { api, toApiError, type ApiError } from "../../lib/api";

export type ConnectState =
  | { kind: "idle" }
  | { kind: "starting" }
  | { kind: "waiting"; code: string; expiresAt: number }
  | { kind: "expired" }
  | { kind: "denied" }
  | { kind: "error"; error: ApiError };

/** 연결 흐름 하나. 컴포넌트가 한 번 만들어 계속 쓴다. */
function createController(setState: (state: ConnectState) => void) {
  let connected: (login: string) => void = () => undefined;
  let active = false;
  let timer: ReturnType<typeof setTimeout> | null = null;
  const clear = () => {
    if (timer !== null) clearTimeout(timer);
    timer = null;
  };
  const schedule = (secs: number) => {
    timer = setTimeout(() => void poll(), secs * 1000);
  };
  const poll = async (): Promise<void> => {
    if (!active) return;
    clear();
    try {
      const result = await api.githubConnectPoll();
      if (!active) return;
      switch (result.kind) {
        case "pending":
          schedule(result.intervalSecs);
          break;
        case "connected":
          active = false;
          setState({ kind: "idle" });
          connected(result.login);
          break;
        case "expired":
        case "denied":
          active = false;
          setState({ kind: result.kind });
          break;
      }
    } catch (error) {
      active = false;
      setState({ kind: "error", error: toApiError(error) });
    }
  };
  const start = async (workflow = false) => {
    clear();
    setState({ kind: "starting" });
    try {
      const challenge = await api.githubConnectStart(workflow);
      active = true;
      setState({ kind: "waiting", code: challenge.userCode, expiresAt: Date.now() + challenge.expiresInSecs * 1000 });
      // 코드를 넣는 페이지(주소는 core 상수). 열지 못해도 코드는 보인다.
      void api.openHostingPage(null, { kind: "deviceLogin" }).catch(() => undefined);
      schedule(challenge.intervalSecs);
    } catch (error) {
      active = false;
      setState({ kind: "error", error: toApiError(error) });
    }
  };
  const stop = (notify: boolean) => {
    const was = active;
    active = false;
    clear();
    if (was) void api.githubConnectCancel().catch(() => undefined);
    if (notify) setState({ kind: "idle" });
  };
  return {
    /** 연결되면 부를 함수(화면이 그릴 때마다 바뀔 수 있다). */
    listen: (fn: (login: string) => void) => {
      connected = fn;
    },
    start,
    cancel: () => stop(true),
    dispose: () => stop(false),
    pollNow: () => {
      if (active) void poll();
    },
  };
}

/**
 * GitHub 연결(device flow, D-121). 시작하면 core가 준 코드를 보이고 기본 브라우저로 코드 페이지를
 * 연다. core가 준 간격마다 묻고(간격은 core도 지킨다), 창으로 돌아오면 바로 한 번 묻는다.
 * 화면을 닫으면(언마운트) 멈추고 core에 취소를 알린다.
 */
export function useConnect(onConnected: (login: string) => void) {
  const [state, setState] = useState<ConnectState>({ kind: "idle" });
  const [controller] = useState(() => createController(setState));
  useEffect(() => controller.listen(onConnected), [controller, onConnected]);
  useEffect(() => {
    window.addEventListener("focus", controller.pollNow);
    return () => {
      window.removeEventListener("focus", controller.pollNow);
      controller.dispose();
    };
  }, [controller]);
  return { state, start: controller.start, cancel: controller.cancel };
}
