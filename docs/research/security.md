# 보안과 신뢰성 대책

- 날짜: 2026-10-15
- 상태: 조사 결과. 채택하면 `docs/decision.md`에 결정으로 올리고, 지킬 규칙은 `docs/architecture.md`와 code-analyst 점검표에 옮긴다.
- 범위: hiGit이 다루는 민감정보, Git 고유의 위협, Tauri 앱 자체, 코드 작성 규칙, 공급망과 배포, 신뢰성.

## 원칙
1. 비밀은 OS가 지킨다. 토큰과 비밀번호는 hiGit 파일이나 설정, 로그에 두지 않고 OS 키체인에만 둔다.
2. 저장소 안의 내용은 믿지 않는다. 남이 만든 저장소의 설정, 파일 이름, 브랜치 이름, 커밋 메시지, 파일 내용은 모두 입력값이다.
3. 웹뷰는 권한이 없다. 화면 쪽은 정해진 명령만 부를 수 있고, 경로나 명령줄을 직접 넘기지 못한다.
4. 되돌릴 수 있게 한다. 지우거나 덮어쓰는 일은 먼저 백업 ref와 journal을 남긴다.
5. 밖으로 보내지 않는다. 기본값은 사용 통계 없음, 오류 보고는 사용자가 켤 때만.

## 위협 모델
| 공격자나 사고 | 노리는 것 | 주된 대책 |
| --- | --- | --- |
| 악성 저장소를 복제하거나 받은 zip을 연다 | 저장소 설정, 서브모듈, 심볼릭 링크로 명령 실행 | 최소 Git 버전, 신뢰 확인, 위험한 설정 끄기 |
| 이상한 브랜치 이름이나 파일 이름 | git 옵션으로 해석되게 만들기 | `--end-of-options`와 `--`, 이름 검사 |
| 저장소 파일 내용에 섞인 HTML이나 스크립트 | 웹뷰에서 실행되어 IPC 호출 | 이스케이프, CSP, 좁은 capability |
| 같은 PC의 다른 사용자나 악성 프로그램 | 토큰, 설정 파일 | OS 키체인, 평문 저장 금지 |
| 오염된 의존성 | 빌드 결과물에 코드 심기 | lockfile, 감사 도구, 설치 스크립트 제한 |
| 사용자의 실수 | 비밀 파일을 버전에 넣어 올리기 | 버전 저장 전 경고 |
| 중간에 끊기는 작업 | 저장소 깨짐, index.lock 남음 | 시간 제한, 취소, 백업 ref, journal |

## 민감정보 목록
| 정보 | 어디에 생기나 | 처리 |
| --- | --- | --- |
| GitHub 토큰 | GitHub 연결 | OS 키체인에만 저장. 메모리에서는 `secrecy` 타입으로 감싸 로그와 Debug 출력에 안 나오게 |
| HTTPS 아이디와 비밀번호 | 다른 원격 저장소 | hiGit이 보관하지 않고 Git의 credential helper에 맡김. Windows는 Git Credential Manager가 기본 |
| SSH 키와 암호 | SSH 원격 | 키 파일을 읽거나 옮기지 않음. ssh-agent와 사용자의 ssh 설정을 그대로 씀 |
| 원격 주소 속 토큰 | `https://user:token@github.com/...` 형태 주소 | 화면, 로그, 오류 문구에서 `https://***@github.com/...`로 가림. 이런 주소를 새로 만들지 않음 |
| 이름과 이메일 | 버전 저장 | 사용자 정보로 다루되 비밀은 아님. 로그와 오류 보고에서는 뺌. GitHub의 noreply 이메일 안내 |
| 저장소 경로와 파일 이름 | 최근 프로젝트 목록, 로그 | 앱 데이터 폴더에만 저장. 오류 보고에는 경로를 가림 |
| 저장소 안의 비밀 | `.env`, 키 파일, 코드 속 토큰 | 버전 저장 전 경고. 아래 "비밀을 올리지 않게" 참고 |
| 앞으로 붙일 연동의 토큰 | 이슈 관리, 다른 호스팅 | GitHub 토큰과 같은 방식. hosting crate가 키체인 접근을 한 곳으로 모음 |

## 인증 정보 보관
GitHub 연결은 OAuth device flow로 한다. 앱이 코드를 보여주면 사용자가 브라우저에서 github.com/login/device에 넣는 방식이라, 데스크톱 앱에 client secret을 넣지 않아도 된다. 권한 범위는 필요한 만큼만 받는다. 1차는 `repo`와 `read:user`, `user:email` 정도이고, 나중에 GitHub App의 세분화된 권한으로 옮길 수 있게 hosting crate 안에 감춘다.

토큰은 Rust `keyring` crate로 OS 키체인에 저장한다. Windows는 자격 증명 관리자, Ubuntu는 Secret Service를 통해 GNOME 키링에 들어간다. Secret Service가 없는 환경에서는 평문 파일로 내려가지 않고, "이 컴퓨터에서는 로그인을 기억할 수 없어요"라고 알린 뒤 실행할 때마다 연결하게 한다.

git에 토큰을 넘길 때는 명령줄이나 주소에 넣지 않는다. 명령줄은 다른 프로세스가 볼 수 있고 셸 기록과 오류 문구에 남기 때문이다. hiGit이 credential helper 역할을 하는 작은 실행 파일을 같이 두고, 해당 git 호출에만 `-c credential.helper=` 로 기존 helper를 비운 뒤 hiGit helper를 지정한다. helper는 키체인에서 토큰을 읽어 표준 출력으로만 건넨다. 사용자가 이미 Git Credential Manager를 쓰고 있으면 그대로 두는 것이 기본이다.

토큰이 만료되거나 취소되면 401을 받는 순간 키체인 항목을 지우고 "GitHub 연결이 끊겼어요. 다시 연결"로 안내한다. 연결 끊기 메뉴는 키체인 항목을 지우고 GitHub 설정의 앱 권한 페이지 링크를 함께 준다.

## Git 고유의 위협
### 최소 Git 버전
시작할 때 `git --version`을 읽어 보안 수정이 들어간 버전보다 낮으면 경고한다. 기준이 되는 사례는 다음과 같다.

| 취약점 | 내용 | 고친 버전 |
| --- | --- | --- |
| CVE-2024-32002 | 서브모듈과 대소문자를 가리지 않는 파일 시스템의 심볼릭 링크로 복제 중 코드 실행. Windows가 해당 | 2.45.1, 2.44.1, 2.43.4, 2.42.2, 2.41.1, 2.40.2, 2.39.4 |
| CVE-2024-32004, 32465 | 로컬 복제와 받은 저장소로 명령 실행 | 위와 같음 |
| CVE-2025-48384 | 설정 값 끝의 CR 처리로 서브모듈 복제 때 임의 파일 쓰기 | 2.50.1 등 각 계열 패치 |
| CVE-2022-24765 | 공유 폴더에 심은 `.git`의 설정이 남의 git 실행에 적용됨 | 2.35.2부터 safe.directory |

기준 버전은 코드 한 곳의 상수로 두고 Git 보안 공지가 나올 때 올린다. 낮으면 막지는 않고 시작 화면 첫 줄에 "Git을 업데이트해 주세요"와 내려받기 링크를 보여준다. 복제는 기준보다 낮을 때 한 번 더 확인한다.

### 인자 주입
브랜치 이름, 태그 이름, 원격 주소, 파일 경로는 `-`로 시작하면 git 옵션으로 읽힐 수 있다. `--upload-pack`이나 `-c core.sshCommand=`처럼 명령을 실행시키는 옵션이 있어서 위험하다.
- 리비전 앞에는 `--end-of-options`, 경로 앞에는 `--`를 붙인다. `--end-of-options`는 대부분 명령에서 2.24부터 쓰이고 rev-parse는 2.30, checkout과 reset은 2.43.1부터 제대로 동작한다. 최소 버전을 올려두면 이 문제도 같이 풀린다.
- 새 브랜치와 태그 이름은 `git check-ref-format`으로 검사하고 `-`로 시작하는 이름은 막는다.
- 원격 주소는 `https://`, `ssh://`, `git@host:path`만 받는다. `ext::`, `file://`, 로컬 경로 복제는 고급 설정에서만 켠다. 모든 git 호출에 `GIT_ALLOW_PROTOCOL=https:ssh:git`을 건다.
- GitBackend는 인자를 배열로 넘기고 셸을 거치지 않는다. 문자열을 이어 붙여 명령줄을 만드는 코드는 금지한다.

### 믿을 수 없는 저장소 설정
`.git/config`는 복제로는 넘어오지 않지만, zip으로 받거나 USB나 공유 폴더로 옮긴 저장소에는 그대로 들어 있다. 그 안의 `core.fsmonitor`, `core.hooksPath`, `core.sshCommand`, `diff.external`, `filter.*.clean`과 `smudge`, `credential.helper`, `core.editor`, `gpg.program`, `uploadpack.packObjectsHook`은 git을 실행하는 순간 프로그램을 돌릴 수 있다.

VS Code의 작업 공간 신뢰처럼 처음 여는 저장소마다 신뢰 여부를 둔다.
- 처음 열 때 위 설정이 로컬 설정에 있는지 `git config --local --list`로 확인한다. 없으면 묻지 않고 신뢰한다.
- 있으면 "이 프로젝트에는 파일을 열 때 실행되는 설정이 있어요"와 설정 이름을 보여주고 "믿고 열기"와 "끄고 열기"를 고르게 한다.
- "끄고 열기"는 모든 호출에 `-c core.fsmonitor=false -c core.hooksPath=<빈 폴더> -c diff.external= -c core.sshCommand=ssh` 같은 덮어쓰기를 붙인다.
- 신뢰 결정은 앱 데이터에 저장소 경로와 설정 내용의 해시로 남기고, 설정이 바뀌면 다시 묻는다.

watcher가 자주 부르는 Query는 신뢰와 상관없이 늘 `--no-ext-diff`, `--no-textconv`, `GIT_OPTIONAL_LOCKS=0`을 쓴다.

"dubious ownership" 오류가 나면 safe.directory를 `*`로 열지 않는다. "이 폴더의 주인이 다른 사용자예요"라고 알리고, 사용자가 고르면 그 경로 하나만 추가한다.

### git 실행 환경
모든 git 호출에 같은 환경을 건다.

| 변수 | 값 | 이유 |
| --- | --- | --- |
| `GIT_TERMINAL_PROMPT` | `0` | 보이지 않는 터미널에서 비밀번호를 기다리며 멈추는 일 방지 |
| `GIT_PAGER`, `PAGER` | `cat` | 페이저 실행 방지 |
| `GIT_EDITOR` | hiGit 지정 값 | 편집기가 뜨며 멈추는 일 방지. 리베이스는 `GIT_SEQUENCE_EDITOR`로 따로 |
| `GIT_ALLOW_PROTOCOL` | `https:ssh:git` | 위험한 전송 방식 차단 |
| `LC_ALL` | `C` | 출력 해석을 언어에 상관없이 고정. 화면 문구는 hiGit이 번역 |
| `GIT_OPTIONAL_LOCKS` | `0` | 읽기 명령이 index.lock을 잡지 않게. Query에만 |

git 실행 파일은 시작할 때 절대 경로로 한 번 찾아 그 경로만 쓴다. Windows에서 저장소 폴더에 심은 `git.exe`가 대신 실행되는 일을 막는다. Rust는 1.77.2 이상을 쓴다. 그 아래는 Windows에서 `.bat` 인자 처리 취약점 CVE-2024-24576이 있다.

### 경로와 심볼릭 링크
화면이 넘기는 경로는 저장소 루트 기준 상대 경로만 받는다. 백엔드에서 정규화한 뒤 루트 밖이면 거부한다. 파일 내용을 읽을 때 심볼릭 링크는 따라가지 않고 "링크: 대상 경로"로 보여준다. 1a에서 고친 diff 보기 경로 문제가 이 규칙의 첫 사례다. 테스트에 `../`, 절대 경로, Windows 드라이브 문자, 링크가 루트 밖을 가리키는 경우를 넣는다.

## Tauri와 웹뷰
| 항목 | 할 일 |
| --- | --- |
| capability | 창마다 필요한 명령만 허용. 기본 `fs`, `shell` 플러그인은 넣지 않고 파일과 git 일은 hiGit 자체 명령으로만 |
| 명령 인자 | 화면은 경로 대신 저장소 ID를 넘기고 백엔드가 등록된 경로로 바꿈. 모든 인자를 serde 타입으로 받고 길이와 형식을 검사 |
| CSP | `default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: asset:; connect-src ipc: http://ipc.localhost`. 원격 스크립트와 iframe 금지 |
| 원격 콘텐츠 | 웹뷰에 외부 페이지를 열지 않음. 링크는 opener 플러그인으로 https만 기본 브라우저에서 |
| 개발자 도구 | 릴리스 빌드에서 끔 |
| `withGlobalTauri` | 끔. 화면 코드는 모듈 import로만 API 사용 |
| 업데이트 | Tauri updater 서명 필수. 서명 키는 CI 비밀과 오프라인 백업 두 곳에만 |
| 단일 실행 | single-instance 플러그인으로 두 번째 실행은 첫 창에 경로만 넘김. 같은 저장소에 두 앱이 동시에 쓰는 일 방지 |

화면에 나오는 파일 내용, 커밋 메시지, 브랜치 이름은 모두 글자로만 그린다. React의 기본 이스케이프를 쓰고 `dangerouslySetInnerHTML`과 `innerHTML`은 ESLint로 막는다. 나중에 검토 요청 설명처럼 마크다운을 그려야 하면 DOMPurify로 거른 뒤 그리고, 원격 이미지는 불러오지 않는다. diff 하이라이트는 토큰을 span으로 만드는 방식만 쓴다.

## 비밀을 올리지 않게
초보자가 가장 자주 내는 사고는 `.env`나 키 파일을 버전에 넣어 GitHub에 올리는 일이다. 버전 저장과 올리기 전에 이번 버전에 넣은 파일을 검사한다.

| 검사 | 예 |
| --- | --- |
| 파일 이름 | `.env`, `.env.*`, `id_rsa`, `id_ed25519`, `*.pem`, `*.key`, `*.p12`, `*.pfx`, `*.keystore`, `credentials.json`, `service-account*.json` |
| 내용 패턴 | `-----BEGIN ... PRIVATE KEY-----`, `ghp_`, `gho_`, `github_pat_`, `AKIA`로 시작하는 AWS 키, `xox[bp]-` Slack 토큰, `sk-`로 시작하는 API 키 |
| 크기 | 큰 파일은 GitHub 제한 100MB 전에 경고 |

규칙은 gitleaks 기본 규칙 중 오탐이 적은 것만 골라 core crate에 데이터로 둔다. 걸리면 막지 않고 "비밀번호나 키로 보이는 파일이 있어요"와 함께 "이번 버전에서 빼기", "제외할 파일에 추가", "그래도 저장"을 보여준다. 이미 올린 뒤라면 기록을 지우는 것보다 키를 새로 발급하는 것이 먼저라고 안내한다. 새 저장소를 만들 때는 `.env`, 키 파일, OS 잡파일이 들어간 기본 제외할 파일 목록을 제안한다.

## 코드 작성 규칙
Rust
- 모든 crate에 `#![forbid(unsafe_code)]`. Windows API가 꼭 필요한 platform crate만 `deny`로 두고 쓰는 곳마다 이유를 주석으로.
- clippy에서 `unwrap_used`, `expect_used`, `panic`을 테스트 밖에서 경고 이상으로. 오류는 `thiserror` 타입으로 올리고 화면 문구는 glossary로 바꿈.
- 토큰은 `secrecy::SecretString`으로만 다루고, 오류 타입과 로그에 넣지 않음.
- porcelain 출력, 설정, diff를 읽는 파서는 `-z` 출력만 쓰고 `cargo-fuzz`로 퍼징. 이상한 바이트가 와도 패닉 없이 오류로 끝나야 함.
- 파일 이름은 UTF-8이 아닐 수 있으니 `OsString`과 바이트로 다루고 화면에 보낼 때만 손실 변환.

TypeScript
- `strict`, `noUncheckedIndexedAccess`. IPC 응답은 생성된 타입으로 받음.
- `eval`, `new Function`, `dangerouslySetInnerHTML` 금지 규칙.

## 공급망과 배포
| 항목 | 할 일 |
| --- | --- |
| Rust 의존성 | `cargo-deny`로 취약점, 라이선스, 출처, 중복 검사. CI에서 매번 |
| JS 의존성 | pnpm lockfile 고정, CI는 `--frozen-lockfile`. 설치 스크립트는 `onlyBuiltDependencies`에 적은 것만 허용 |
| 업데이트 | Dependabot이나 Renovate. 새 버전은 며칠 뒤에 받도록 지연 설정 |
| GitHub Actions | 액션을 커밋 SHA로 고정. 워크플로 기본 권한 `contents: read`, 필요한 작업에만 더 줌. `pull_request_target` 쓰지 않음 |
| 릴리스 | CI에서만 빌드. 로컬 빌드를 배포하지 않음. SHA256 체크섬 같이 게시 |
| 코드 서명 | Windows는 Authenticode 서명. 없으면 SmartScreen 경고가 떠서 비개발자가 설치를 포기함. Linux는 deb와 AppImage에 GPG 서명 |
| 비밀 | 서명 키와 토큰은 GitHub Environments 비밀로, 릴리스 작업에서만 읽힘 |

## 신뢰성
| 상황 | 대책 |
| --- | --- |
| git이 멈춤 | 명령 종류별 시간 제한. Query는 짧게, 네트워크는 길게 하되 진행이 멈추면 끊음 |
| 사용자가 취소 | 프로세스 트리째 종료. Windows는 Job Object, Linux는 프로세스 그룹 |
| index.lock이 남음 | 바로 지우지 않음. 살아 있는 git 프로세스가 없고 일정 시간이 지났을 때만 "멈춘 작업 정리"로 묻고 지움 |
| 지우거나 덮어쓰는 Action | 먼저 백업 ref와 journal. 복구 화면에서 되돌릴 수 있게 |
| 앱 설정 저장 | 임시 파일에 쓰고 rename. 쓰다 꺼져도 이전 설정이 남음 |
| 디스크 가득 참, 권한 없음 | 쉬운 문구로 원인과 할 일. 원래 상태가 바뀌지 않았음을 확인해 알림 |
| 큰 저장소 | 목록과 노선도는 나눠 읽기, 큰 파일 diff는 상한 |
| 앱이 죽음 | 다음 실행에서 journal을 보고 끝나지 않은 작업을 알림 |

## 개인정보와 로그
로그는 앱 데이터 폴더에만 남기고 크기와 기간을 정해 순환한다. 로그에 쓰기 전에 한 곳에서 가리는 함수를 거친다. 주소 속 아이디와 비밀번호, 위 토큰 패턴, 이메일, 홈 폴더 경로를 가린다. 사용 통계는 넣지 않는다. 오류 보고는 설정에서 켤 때만 보내고, 보내기 전에 내용을 보여준다.

## 절차
- 저장소 루트에 `SECURITY.md`. 취약점 신고 방법과 GitHub private vulnerability reporting 사용.
- 이 문서의 위협 모델을 `docs/architecture.md`에 짧게 옮기고, 새 Action과 새 IPC 명령을 만들 때 해당 줄을 확인.
- code-analyst 점검표에 아래 항목을 추가.

```
보안 점검
- git 호출: 인자 배열, 리비전 앞 --end-of-options, 경로 앞 --, 공통 환경 변수
- IPC 명령: 저장소 ID로 받기, 경로는 루트 안 상대 경로만, 인자 형식 검사
- 비밀: 토큰이 로그, 오류, Debug, 명령줄, 주소에 들어가지 않음
- 화면: innerHTML 없음, 저장소에서 온 글자는 그대로 그리기
- 지우거나 덮어쓰는 일: 백업 ref와 journal 먼저
- unsafe, unwrap 새로 생기지 않음
- 새 의존성: 필요성, 관리 상태, cargo-deny 통과
```

## 언제 넣나
| 시기 | 항목 |
| --- | --- |
| 1b 지금 | git 실행 환경, `--end-of-options`와 `--`, 경로 정규화와 링크 테스트, 최소 Git 버전 경고, CSP와 capability 점검, innerHTML 금지 규칙, forbid unsafe, cargo-deny와 pnpm 고정, Actions 권한, code-analyst 점검표 |
| 1b 끝나기 전 | 저장소 신뢰 확인, 비밀 파일 경고, 로그 가리기, 시간 제한과 취소, index.lock 처리 |
| GitHub 연결 단계 | device flow, keyring, credential helper, 토큰 만료 처리 |
| 첫 배포 전 | 코드 서명, updater 서명, SECURITY.md, 파서 퍼징, 체크섬 |

## 출처
- Git CVE-2024-32002 공지: https://github.com/git/git/security/advisories/GHSA-8h77-4q3w-gfgv
- Git CVE-2022-24765와 safe.directory: https://github.blog/2022-04-12-git-security-vulnerability-announced/
- Git 2.50.1 등 2025년 7월 보안 릴리스: https://github.blog/open-source/git/git-security-vulnerabilities-announced-6/
- `--end-of-options`: https://git-scm.com/docs/gitcli
- Tauri capability: https://v2.tauri.app/security/capabilities/
- Tauri CSP: https://v2.tauri.app/security/csp/
- GitHub device flow: https://docs.github.com/en/apps/oauth-apps/building-oauth-apps/authorizing-oauth-apps#device-flow
- keyring crate: https://docs.rs/keyring
- Rust CVE-2024-24576: https://blog.rust-lang.org/2024/04/09/cve-2024-24576.html
- gitleaks 규칙: https://github.com/gitleaks/gitleaks
- cargo-deny: https://embarkstudios.github.io/cargo-deny/
