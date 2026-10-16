// 옛 앱이 웹뷰 저장소에 남긴 충돌 선택(`higit.conflict.*`)을 한 번 지운다(D-211). 화면에서 웹뷰 저장소를
// 쓰는 곳은 이 파일뿐이다(ESLint가 막는다).

const LEGACY_PREFIX = "higit.conflict.";

/** 앱 시작 때 부른다. 저장소를 못 쓰면(막힌 웹뷰) 아무것도 하지 않는다. */
export function clearLegacyStorage(): void {
  try {
    const store = window.localStorage;
    const keys: string[] = [];
    for (let i = 0; i < store.length; i++) {
      const key = store.key(i);
      if (key?.startsWith(LEGACY_PREFIX)) keys.push(key);
    }
    for (const key of keys) store.removeItem(key);
  } catch {
    // 무시
  }
}
