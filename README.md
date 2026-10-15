# hiGit

Git을 모르는 사람도 쉽게 쓰는 데스크톱 Git 앱입니다. Tauri 2(Rust) + React/TypeScript로 만들고 Windows와 Ubuntu를 지원합니다.

무엇을 왜 만드는지는 `docs/`를 보세요. 개발 규칙은 `CLAUDE.md`에 있습니다.

## 개발 환경 준비

버전은 저장소에 고정되어 있습니다(D-013).

| 도구 | 버전 | 고정 위치 |
| --- | --- | --- |
| Rust | 1.99.0 | `rust-toolchain.toml` (rustup이 자동 설치) |
| Node | 22 | `.nvmrc`, `package.json`의 `engines` |
| pnpm | 12.11.2 | `package.json`의 `packageManager` (`corepack enable pnpm`) |
| git | 2.30 이상 | 앱 실행에 필요 |

### Ubuntu 22.04 이상

```sh
sudo apt-get install -y libwebkit2gtk-4.1-dev librsvg2-dev libssl-dev build-essential libxdo-dev libayatana-appindicator3-dev
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none
# Node 22: nvm 등으로 설치한 뒤
corepack enable pnpm
```

### Windows 10/11

- [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)에서 "C++를 사용한 데스크톱 개발" 설치
- WebView2 런타임(Windows 11은 기본 포함)
- [rustup](https://rustup.rs/), Node 22, `corepack enable pnpm`
- [Git for Windows](https://git-scm.com/download/win)

## 실행

```sh
pnpm install
pnpm tauri dev
```

## Windows 시험판 받기

main에 push할 때마다 CI가 Windows 실행 파일을 만들어 올립니다(14일 보관).

1. GitHub의 Actions 탭에서 최근 성공한 CI 실행을 엽니다. 특정 커밋이 필요하면 "Run workflow"로 수동 실행합니다.
2. 아래 Artifacts의 `higit-windows-x64-<커밋>`을 내려받아 압축을 풉니다.
3. `higit.exe`를 실행합니다. 필요한 것(WebView2, Git for Windows)과 경고 대처는 함께 든 `README.txt`에 있습니다.

`gh`가 있으면: `gh run download --name higit-windows-x64-<커밋 전체 해시>`

## 커밋 전 검사

저장소에 들어 있는 git hook(`.githooks/`)이 검사를 강제합니다. `pnpm install` 때 자동으로 켜집니다.

- 커밋할 때: 바뀐 쪽만 빠르게 검사합니다. 프론트는 lint·타입 검사·테스트, Rust는 형식·clippy.
- push할 때: 전체 검사(Rust 테스트, 바인딩이 최신인지 포함).
- 켜졌는지 확인: `git config core.hooksPath` → `.githooks`. 아니면 `pnpm run prepare`.
- 직접 실행: `pnpm check:fast`, `pnpm check:full`.
- hook은 작업 폴더 상태를 검사하므로, 스테이지하지 않은 수정도 결과에 섞입니다.
- Windows: hook은 Git for Windows의 sh로 실행됩니다. 그 환경의 PATH에서 `node`, `pnpm`, `cargo`를 찾을 수 있어야 합니다.

## 서드파티 고지

앱에 포함된 글꼴 등의 라이선스는 `THIRD_PARTY_NOTICES.md`에 있습니다.
