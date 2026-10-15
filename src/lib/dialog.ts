// 폴더 선택 대화상자. git 호출이 아니라서 화면에서 직접 쓰되, 이 파일로만 감싼다.
import { open } from "@tauri-apps/plugin-dialog";

/** 폴더를 고르게 하고 경로를 돌려준다. 취소하면 null. */
export async function pickFolder(): Promise<string | null> {
  const picked = await open({ directory: true, multiple: false });
  return typeof picked === "string" ? picked : null;
}
