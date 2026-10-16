// 클립보드에 글자 넣기(역 메뉴·복제·연결 코드). 웹뷰에 따라 navigator.clipboard가 없을 수 있어 옛 방법으로 한 번 더 해 본다.

export async function copyText(text: string): Promise<boolean> {
  try {
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(text);
      return true;
    }
  } catch {
    // 아래 방법으로 넘어간다.
  }
  const area = document.createElement("textarea");
  area.value = text;
  area.setAttribute("readonly", "");
  area.style.position = "fixed";
  area.style.opacity = "0";
  document.body.append(area);
  area.select();
  try {
    return document.execCommand("copy");
  } catch {
    return false;
  } finally {
    area.remove();
  }
}

/**
 * 클립보드 글자 읽기(입력칸 앱 메뉴의 붙여넣기, D-189). 사람의 동작 안에서만 부른다. 웹뷰가 막거나 없으면
 * `null`이고 화면은 "Ctrl+V를 눌러 주세요"로 안내한다. 읽은 값은 저장하거나 기록하지 않는다.
 */
export async function readText(): Promise<string | null> {
  try {
    if (!navigator.clipboard?.readText) return null;
    return await navigator.clipboard.readText();
  } catch {
    return null;
  }
}

/** 클립보드에서 글자를 읽을 수 있어 보이는지(항목을 미리 끌지 정한다). */
export function canReadText(): boolean {
  return typeof navigator.clipboard?.readText === "function";
}
