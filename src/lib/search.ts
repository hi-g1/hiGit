// 목록 찾기 칸의 글자 맞추기(D-185). 대소문자를 무시하고 한글 조합형·완성형을 같게 본다(NFC).

/** 찾을 글자를 맞추기 좋은 꼴로(앞뒤 공백 없이, NFC, 소문자). */
export function searchKey(text: string): string {
  return text.trim().normalize("NFC").toLocaleLowerCase();
}

/** `text`에 `query`가 들어 있는지. 빈 찾기는 모두 맞는다. */
export function matchesSearch(text: string, query: string): boolean {
  const key = searchKey(query);
  return key === "" || text.normalize("NFC").toLocaleLowerCase().includes(key);
}
