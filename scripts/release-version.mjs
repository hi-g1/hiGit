// 릴리스 태그와 앱 버전이 같은지 본다(D-221). 사용법: node scripts/release-version.mjs [태그]
// 버전을 출력하고, 태그를 주었는데 `v<버전>`이 아니면 실패한다.
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const ROOT = fileURLToPath(new URL("../", import.meta.url));

/** 앱 버전(package.json). 세 곳이 같은지는 version-sync 시험이 본다. */
export function appVersion() {
  return JSON.parse(readFileSync(`${ROOT}package.json`, "utf8")).version;
}

/** 태그가 이 버전의 릴리스 태그인지. */
export function tagMatches(tag, version) {
  return tag === `v${version}`;
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  const version = appVersion();
  const tag = process.argv[2];
  if (tag && !tagMatches(tag, version)) {
    console.error(`태그 ${tag}가 앱 버전 ${version}과 다릅니다(v${version}이어야 합니다).`);
    process.exit(1);
  }
  console.log(version);
}
