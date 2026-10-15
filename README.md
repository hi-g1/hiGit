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

## 서드파티 고지

앱에 포함된 글꼴 등의 라이선스는 `THIRD_PARTY_NOTICES.md`에 있습니다.
