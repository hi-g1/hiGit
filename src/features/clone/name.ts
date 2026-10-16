// 복제 주소에서 폴더 이름 제안. core가 최종 검사하고 여기는 힌트일 뿐이다.

/** 두 OS에서 폴더 이름으로 쓸 수 없는 글자(제어 문자는 아래에서 따로). */
const UNSAFE = /[/\\:*?"<>|]/g;

function dropControls(text: string): string {
  return [...text].map((c) => (c.charCodeAt(0) < 32 || c.charCodeAt(0) === 127 ? "-" : c)).join("");
}

/** `https://github.com/a/b.git` → `b`, `git@github.com:a/b.git` → `b`. 못 구하면 빈 값. */
export function suggestName(url: string): string {
  let rest = url.trim().replace(/\/+$/, "");
  if (rest.toLowerCase().endsWith(".git")) rest = rest.slice(0, -4);
  const last = rest.split(/[/:]/).pop() ?? "";
  const name = dropControls(last)
    .replace(UNSAFE, "-")
    .replace(/\s+/g, " ")
    .replace(/^[\s.-]+|[\s.]+$/g, "");
  return name === "." || name === ".." ? "" : name;
}

/** 부모 폴더와 이름을 그 OS의 구분자로 잇는다. */
export function joinPath(parent: string, name: string): string {
  if (!parent || !name) return parent || name;
  const sep = parent.includes("\\") && !parent.includes("/") ? "\\" : "/";
  return parent.endsWith(sep) ? `${parent}${name}` : `${parent}${sep}${name}`;
}
