import "@testing-library/jest-dom/vitest";
import { cleanup } from "@testing-library/react";
import { afterEach } from "vitest";
import { useUiStore } from "../state/uiStore";

afterEach(() => {
  cleanup();
  // 화면 배율·언어 같은 화면 설정이 다음 시험으로 새지 않게 한다.
  useUiStore.setState(useUiStore.getInitialState());
});

// jsdom에는 포인터 캡처가 없다. Radix Toast의 밀어서 닫기가 부른다.
if (typeof Element !== "undefined" && !Element.prototype.hasPointerCapture) {
  Element.prototype.hasPointerCapture = () => false;
  Element.prototype.setPointerCapture = () => undefined;
  Element.prototype.releasePointerCapture = () => undefined;
}
