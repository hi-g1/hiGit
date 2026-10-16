// 서드파티 라이선스 고지 THIRD_PARTY_LICENSES.txt를 만든다(D-220, 계획 014 R16b).
// 사용법: node scripts/notices.mjs          파일을 다시 쓴다
//         node scripts/notices.mjs --check  목록이 지금 의존성과 같은지만 본다(CI, pre-push)
//
// 대상은 설치 파일에 실제로 들어가는 것: app crate에서 닿는 Rust crate(Windows·Linux 두 대상, 개발·빌드 의존 뺌)와
// package.json `dependencies`에서 닿는 npm 패키지(`pnpm licenses list --prod`). 글꼴은 THIRD_PARTY_NOTICES.md.
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = fileURLToPath(new URL("../", import.meta.url));
const OUT = join(ROOT, "THIRD_PARTY_LICENSES.txt");
const TARGETS = ["x86_64-pc-windows-msvc", "x86_64-unknown-linux-gnu"];
const shell = process.platform === "win32";

function run(cmd, args, cwd) {
  const out = spawnSync(cmd, args, { cwd, encoding: "utf8", shell, maxBuffer: 256 * 1024 * 1024 });
  if (out.status !== 0) {
    console.error(`${cmd} ${args.join(" ")} 실패\n${out.stderr}`);
    process.exit(1);
  }
  return out.stdout;
}

/** app crate에서 일반 의존으로 닿는 외부 crate. */
function rustPackages() {
  const tauri = join(ROOT, "src-tauri");
  const found = new Map();
  for (const target of TARGETS) {
    const meta = JSON.parse(
      run("cargo", ["metadata", "--format-version", "1", "--locked", "--filter-platform", target], tauri),
    );
    const packages = new Map(meta.packages.map((p) => [p.id, p]));
    const nodes = new Map(meta.resolve.nodes.map((n) => [n.id, n]));
    const root = meta.packages.find((p) => p.name === "app" && p.source === null);
    const stack = [root.id];
    const seen = new Set();
    while (stack.length > 0) {
      const id = stack.pop();
      if (seen.has(id)) continue;
      seen.add(id);
      for (const dep of nodes.get(id)?.deps ?? []) {
        if (dep.dep_kinds.some((k) => k.kind === null)) stack.push(dep.pkg);
      }
    }
    for (const id of seen) {
      const p = packages.get(id);
      if (p.source === null) continue;
      found.set(`${p.name} ${p.version}`, {
        kind: "rust",
        name: p.name,
        version: p.version,
        license: p.license ?? (p.license_file ? "파일 참고" : "모름"),
        dir: dirname(p.manifest_path),
        authors: p.authors ?? [],
      });
    }
  }
  return [...found.values()];
}

/** 화면에 묶이는 npm 패키지. */
function npmPackages() {
  const list = JSON.parse(run("pnpm", ["licenses", "list", "--json", "--prod"], ROOT));
  const out = [];
  for (const entries of Object.values(list)) {
    for (const e of entries) {
      e.versions.forEach((version, i) => {
        out.push({
          kind: "npm",
          name: e.name,
          version,
          license: e.license ?? "모름",
          dir: e.paths[i] ?? e.paths[0],
          authors: e.author ? [e.author] : [],
        });
      });
    }
  }
  return out;
}

const LICENSE_FILE = /^(licen[cs]e|copying|notice)/i;

/** 패키지 폴더의 라이선스 파일. 두 라이선스 중 고를 수 있으면 MIT 쪽만 넣는다. NOTICE는 늘 넣는다. */
function licenseTexts(pkg) {
  if (!existsSync(pkg.dir)) return [];
  const files = readdirSync(pkg.dir)
    .filter((f) => LICENSE_FILE.test(f))
    .sort();
  const dual = /\bMIT\b/.test(pkg.license) && /\bOR\b|\//.test(pkg.license);
  const mit = files.filter((f) => /mit/i.test(f));
  const notices = files.filter((f) => /^notice/i.test(f));
  const chosen = dual && mit.length > 0 ? [...mit, ...notices] : files;
  return [...new Set(chosen)].map((f) => readFileSync(join(pkg.dir, f), "utf8").replace(/\r\n/g, "\n").trim());
}

function mitTemplate(pkg) {
  const holders = pkg.authors.length > 0 ? pkg.authors.join(", ") : `${pkg.name} authors`;
  return `MIT License\n\nCopyright (c) ${holders}\n\nPermission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:\n\nThe above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.\n\nTHE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.`;
}

function bsdTemplate(pkg) {
  const holders = pkg.authors.length > 0 ? pkg.authors.join(", ") : `${pkg.name} authors`;
  const three = /BSD-3-Clause/.test(pkg.license);
  const clauses = [
    "1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following disclaimer.",
    "2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the following disclaimer in the documentation and/or other materials provided with the distribution.",
    ...(three
      ? ["3. Neither the name of the copyright holder nor the names of its contributors may be used to endorse or promote products derived from this software without specific prior written permission."]
      : []),
  ];
  return `${three ? "BSD 3-Clause" : "BSD 2-Clause"} License\n\nCopyright (c) ${holders}\n\nRedistribution and use in source and binary forms, with or without modification, are permitted provided that the following conditions are met:\n\n${clauses.join("\n\n")}\n\nTHIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.`;
}

const FONT_NOTE =
  "SIL Open Font License 1.1. 원문과 저작권 표시는 함께 설치되는 licenses/ 폴더와 THIRD_PARTY_NOTICES.md의 글꼴 절에 있습니다.";

/** 로캘과 상관없는 순서(어느 컴퓨터에서 만들어도 같은 파일). */
const cmp = (a, b) => (a < b ? -1 : a > b ? 1 : 0);
const byPackage = (a, b) => cmp(a.kind, b.kind) || cmp(a.name, b.name) || cmp(a.version, b.version);

const line = (p) => `${p.kind.padEnd(5)} ${p.name} ${p.version}  ${p.license}`;

function inventory(packages) {
  return packages
    .slice()
    .sort(byPackage)
    .map(line);
}

function render(packages) {
  const sorted = packages
    .slice()
    .sort(byPackage);
  // 저작권 줄이 없는 원문(Apache-2.0, MPL-2.0)은 파일이 없는 패키지에 다른 패키지의 원문을 쓴다.
  const shared = { "Apache-2.0": /Apache License\s+Version 2\.0/, "MPL-2.0": /Mozilla Public License,? [Vv]ersion 2\.0/ };
  const sharedText = {};
  const texts = new Map();
  const missing = [];
  const add = (p, text) => {
    const key = createHash("sha256").update(text).digest("hex");
    const group = texts.get(key) ?? { text, who: [] };
    group.who.push(`${p.name} ${p.version}`);
    texts.set(key, group);
  };
  for (const p of sorted) {
    let found = licenseTexts(p);
    if (found.length === 0 && /\bMIT\b/.test(p.license)) found = [mitTemplate(p)];
    if (found.length === 0 && /BSD-[23]-Clause/.test(p.license)) found = [bsdTemplate(p)];
    // 글꼴 패키지는 OFL 원문을 묶음의 licenses/ 폴더에 따로 넣는다(THIRD_PARTY_NOTICES.md).
    if (found.length === 0 && p.license === "OFL-1.1") found = [FONT_NOTE];
    if (found.length === 0) {
      missing.push(p);
      continue;
    }
    for (const [id, re] of Object.entries(shared)) sharedText[id] ??= found.find((t) => re.test(t));
    for (const text of found) add(p, text);
  }
  for (const p of missing) {
    const id = Object.keys(shared).find((k) => p.license.includes(k) && sharedText[k]);
    if (!id) {
      console.error(`라이선스 원문을 찾지 못했습니다: ${p.kind} ${p.name} ${p.version} (${p.license})`);
      process.exit(1);
    }
    add(p, sharedText[id]);
  }
  const head = [
    "hiGit 서드파티 라이선스",
    "=======================",
    "",
    "hiGit에 함께 들어가는 Rust crate와 화면 라이브러리의 라이선스입니다. 글꼴은 THIRD_PARTY_NOTICES.md에 있습니다.",
    "여러 라이선스 중 고를 수 있는 패키지는 MIT 원문을 넣었습니다.",
    "이 파일은 scripts/notices.mjs가 만듭니다. 고치지 말고 `node scripts/notices.mjs`로 다시 만드세요.",
    "",
    "목록",
    "----",
    ...inventory(packages),
    "",
    "원문",
    "----",
  ];
  const body = [...texts.values()].map((g) => `=== ${g.who.join(", ")} ===\n\n${g.text}\n`);
  return `${head.join("\n")}\n\n${body.join("\n")}`;
}

const check = process.argv.includes("--check");
if (!check) {
  // 다른 OS 대상의 crate 원문도 읽게 모두 받아 둔다(cargo 캐시).
  run("cargo", ["fetch", "--locked"], join(ROOT, "src-tauri"));
}
const packages = [...rustPackages(), ...npmPackages()];
if (check) {
  const want = inventory(packages);
  const have = existsSync(OUT) ? readFileSync(OUT, "utf8").replace(/\r\n/g, "\n") : "";
  const start = have.indexOf("목록\n----\n");
  const end = have.indexOf("\n\n원문\n");
  const listed = start >= 0 && end > start ? have.slice(start + "목록\n----\n".length, end).split("\n") : [];
  const added = want.filter((l) => !listed.includes(l));
  const removed = listed.filter((l) => !want.includes(l));
  if (added.length > 0 || removed.length > 0) {
    console.error("THIRD_PARTY_LICENSES.txt가 지금 의존성과 다릅니다. `node scripts/notices.mjs`로 다시 만드세요.");
    for (const l of added) console.error(`  + ${l}`);
    for (const l of removed) console.error(`  - ${l}`);
    process.exit(1);
  }
  console.log(`서드파티 고지 최신(${want.length}개)`);
} else {
  writeFileSync(OUT, render(packages));
  console.log(`THIRD_PARTY_LICENSES.txt를 썼습니다(${packages.length}개)`);
}
