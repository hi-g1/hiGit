// 동시에 도는 작업 수를 묶는다(시작 화면 노선도를 한꺼번에 부르지 않게, D-093).

export type Limiter = (<T>(task: () => Promise<T>) => Promise<T>) & { reset: () => void };

export function limiter(max: number): Limiter {
  let running = 0;
  let waiting: (() => void)[] = [];
  const run = async <T>(task: () => Promise<T>): Promise<T> => {
    while (running >= max) await new Promise<void>((resolve) => waiting.push(resolve));
    running += 1;
    try {
      return await task();
    } finally {
      running -= 1;
      waiting.shift()?.();
    }
  };
  /** 테스트끼리 상태가 남지 않게 비운다. */
  run.reset = () => {
    running = 0;
    waiting = [];
  };
  return run;
}
