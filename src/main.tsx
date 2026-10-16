import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App, createQueryClient } from "./App";
import { preloadDisplay } from "./lib/preload";
import { installWebFeel } from "./lib/webFeel";
import "./styles/fonts.css";
import "./styles/tokens.css";
import "./styles/base.css";

const root = document.getElementById("root");
if (!root) throw new Error("#root 요소가 없습니다");

installWebFeel({ prod: import.meta.env.PROD });

// 글자 크기를 정한 뒤 그린다(D-149). 읽지 못해도 기본값으로 그린다.
const client = createQueryClient();
void preloadDisplay(client).finally(() => {
  createRoot(root).render(
    <StrictMode>
      <App client={client} />
    </StrictMode>,
  );
});
