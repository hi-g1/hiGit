// 한글 찾기(D-199): 초성 찾기("ㅂㄹㅊ" → "브랜치")와 섞인 찾기("버ㅈ" → "버전 저장"). 순수 함수라 DOM 없이 시험한다.

const SYLLABLE_FIRST = 0xac00;
const SYLLABLE_LAST = 0xd7a3;
/** 초성 순서(유니코드 음절 표의 순서). */
const CHOSEONG = ["ㄱ", "ㄲ", "ㄴ", "ㄷ", "ㄸ", "ㄹ", "ㅁ", "ㅂ", "ㅃ", "ㅅ", "ㅆ", "ㅇ", "ㅈ", "ㅉ", "ㅊ", "ㅋ", "ㅌ", "ㅍ", "ㅎ"];
const CHOSEONG_SET = new Set(CHOSEONG);
/** 조합형 초성(U+1100~U+1112)은 같은 순서다. */
const CONJOINING_FIRST = 0x1100;

function isSyllable(code: number): boolean {
  return code >= SYLLABLE_FIRST && code <= SYLLABLE_LAST;
}

/** 조합 자모를 호환 자모로, 풀어 쓴 음절은 모아 쓴다(NFC). 입력기마다 다른 꼴을 같게 맞춘다. */
export function normalizeJamo(text: string): string {
  return Array.from(text.normalize("NFC"), (ch) => {
    const code = ch.codePointAt(0) ?? 0;
    const index = code - CONJOINING_FIRST;
    return index >= 0 && index < CHOSEONG.length ? CHOSEONG[index]! : ch;
  }).join("");
}

/** 음절은 초성으로 바꾸고 그 밖의 글자는 그대로 둔다. */
export function choseong(text: string): string {
  return Array.from(text, (ch) => {
    const code = ch.codePointAt(0) ?? 0;
    return isSyllable(code) ? CHOSEONG[Math.floor((code - SYLLABLE_FIRST) / 588)]! : ch;
  }).join("");
}

/** 받침 없는 음절이면 (초성, 중성) 번호. 조합 중인 마지막 글자("저")가 "전"에도 맞게 쓴다. */
function openSyllable(ch: string): number | null {
  const code = ch.codePointAt(0) ?? 0;
  if (!isSyllable(code) || (code - SYLLABLE_FIRST) % 28 !== 0) return null;
  return Math.floor((code - SYLLABLE_FIRST) / 28);
}

function sameChar(q: string, t: string, last: boolean): boolean {
  if (CHOSEONG_SET.has(q)) return choseong(t) === q;
  if (q === t || q.toLowerCase() === t.toLowerCase()) return true;
  if (!last) return false;
  const open = openSyllable(q);
  const code = t.codePointAt(0) ?? 0;
  return open !== null && isSyllable(code) && Math.floor((code - SYLLABLE_FIRST) / 28) === open;
}

/** 띄어쓰기를 빼고 글자마다(음절, 초성, 섞임) 이어서 맞는 곳이 있는지. */
function jamoMatch(query: string, target: string): boolean {
  const q = Array.from(query.replace(/\s+/g, ""));
  const t = Array.from(target.replace(/\s+/g, ""));
  if (q.length === 0 || q.length > t.length) return false;
  for (let start = 0; start + q.length <= t.length; start++) {
    if (q.every((ch, i) => sameChar(ch, t[start + i]!, i === q.length - 1))) return true;
  }
  return false;
}

/**
 * 찾기 점수. 3: 이름에 그대로 들어 있음, 2: 이름에 초성·섞임으로 맞음, 1: 다른 글자(원래 이름·설명)에 맞음,
 * 0: 맞지 않음. 빈 질의는 모두 1.
 */
export function matchQuery(query: string, name: string, others: readonly string[] = []): number {
  const q = normalizeJamo(query).trim();
  if (q === "") return 1;
  const lower = q.toLowerCase();
  if (name.toLowerCase().includes(lower)) return 3;
  if (jamoMatch(q, name)) return 2;
  if (others.some((o) => o.toLowerCase().includes(lower) || jamoMatch(q, o))) return 1;
  return 0;
}
