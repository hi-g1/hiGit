// 한국어 조사 고르기. 용어(glossary 쉬운 이름)를 문장에 끼울 때 받침에 맞는 조사를 붙인다.

type Pair = "이/가" | "을/를" | "은/는" | "과/와" | "이에요/예요" | "으로/로";

/** 마지막 글자의 받침 번호(0이면 받침 없음). 한글이 아니면 0. */
function finalConsonant(word: string): number {
  const code = word.trimEnd().charCodeAt(word.trimEnd().length - 1);
  if (code < 0xac00 || code > 0xd7a3) return 0;
  return (code - 0xac00) % 28;
}

/** `josa("브랜치", "이/가")` → "브랜치가" */
export function josa(word: string, pair: Pair): string {
  const [withBatchim, without] = pair.split("/") as [string, string];
  const fc = finalConsonant(word);
  // "으로/로"는 ㄹ 받침(8) 뒤에도 "로"
  const useFirst = pair === "으로/로" ? fc !== 0 && fc !== 8 : fc !== 0;
  return word + (useFirst ? withBatchim : without);
}
