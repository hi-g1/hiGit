// 사용법 문서를 그림까지 한 파일로 묶는다(계획 014 R20b). 인터넷 없이 열려야 해서 그림은 data URI로 넣는다.
// 사용법: node scripts/guide-build.mjs  → docs/guide/src/{index,en}.html을 읽어 docs/guide/{index,en}.html을 쓴다.
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const GUIDE = fileURLToPath(new URL("../docs/guide/", import.meta.url));
const TYPES = { webp: "image/webp", png: "image/png", jpg: "image/jpeg" };

for (const name of ["index.html", "en.html"]) {
  const source = join(GUIDE, "src", name);
  if (!existsSync(source)) continue;
  const html = readFileSync(source, "utf8").replace(/src="img\/([\w.-]+)\.(webp|png|jpg)"/g, (_, file, ext) => {
    const bytes = readFileSync(join(GUIDE, "src", "img", `${file}.${ext}`));
    return `src="data:${TYPES[ext]};base64,${bytes.toString("base64")}"`;
  });
  if (/src="(?!data:)[^"]+"/.test(html)) {
    console.error(`${name}: 묶지 못한 외부 그림이 있습니다.`);
    process.exit(1);
  }
  writeFileSync(join(GUIDE, name), html);
  console.log(`${name} ${(html.length / 1024).toFixed(0)}KB`);
}
