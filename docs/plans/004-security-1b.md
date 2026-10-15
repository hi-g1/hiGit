# 004 보안 1b 단계
- 관련 결정: D-034, D-035, D-037, D-038 (확정). D-036 (제안, 확정 전에는 거부 기준을 바꾸지 않는다). 기대는 확정 결정: D-002, D-003, D-010, D-019, D-021, D-023, D-030, D-032. D-039~D-041은 "1b 끝나기 전" 표에서만 다룬다.
- 관련 명세: spec.md "오류 메시지 원칙". 용어표는 건드리지 않는다.
- 참고 자료(결정 아님): `docs/research/security.md` "언제 넣나" 표의 "1b 지금" 행.
- 앞선 계획: 001(1b 남은 단계 6, 8, 9, 10 브랜치 조회, 10b, 12, 13, 19, 20), 003(시작 화면, 구현됨). 다음 계획 005(홈 화면)가 이 계획과 001 1b 단계를 한 순서로 합친다.

## 목표
남이 만든 저장소의 브랜치 이름, 파일 이름, 설정이 git 옵션이나 웹뷰 스크립트로 바뀌어 실행되지 않는다. git이 보이지 않는 곳에서 질문·편집기·페이저로 멈추지 않는다. 오래된 Git이면 시작할 때 알려준다. 의존성과 CI가 기본값으로 좁게 묶인다.

## 현재 상태 ("1b 지금" 항목별)
| 항목 | 상태 | 근거 | 남은 일 → 단계 |
| --- | --- | --- | --- |
| git 실행 환경 | 일부 됨 | `LC_ALL=C`, `LANGUAGE` 지움, `GIT_TERMINAL_PROMPT=0`, `GIT_PAGER=cat`, `GIT_OPTIONAL_LOCKS=0`: `crates/git/src/cli/runner.rs:79-83`. 셸 없이 인자 배열: `runner.rs:73-86`. 조회의 `--no-ext-diff --no-textconv`: `crates/git/src/cli/diff.rs:14-15`. git을 시작할 때 한 번 찾아 경로를 들고 있음: `crates/core/src/bootstrap.rs:48-49`, `crates/platform/src/git_exe.rs:4-8`. Rust 1.99: `rust-toolchain.toml:2`, `src-tauri/Cargo.toml:42` | `PAGER`, `GIT_EDITOR`, `GIT_ALLOW_PROTOCOL` → 1. PATH의 상대 경로 항목 거부 → 2 |
| `--end-of-options`와 `--` | 일부 됨 | 경로 앞 `--`: `crates/git/src/cli/mod.rs:103`, `:144`, `:154`. 경로 목록 stdin + `--literal-pathspecs`: `mod.rs:176-211` | 리비전 앞 `--end-of-options`가 없음(`mod.rs:37`, `:91`, `:103`, `:130`, `:143`) → 3. 옵션처럼 보이는 이름을 core에서 막기 → 4 |
| 경로 정규화와 링크 테스트 | 대부분 됨 | 문자열 검사(절대 경로, 드라이브 문자, `\`, `..`, NUL): `crates/core/src/actions/paths.rs:15-29`, 테스트 `paths.rs:52-69`. 링크가 밖을 가리키면 거부: `crates/core/src/query.rs:109-124`. 통합 테스트(`../`, 밖 링크, 안 링크): `crates/core/tests/query.rs:347-441` | 폴더 링크·Windows 정션 테스트, 링크를 따라가지 않고 "링크: 대상"으로 보여주기 → 5 |
| 최소 Git 버전 경고 | 안 됨 | 거부 기준만 있음: `bootstrap.rs:16`, `:69-71`. 문구에 숫자를 직접 씀: `src/messages/ko.ts:89`, `en.ts:81` | 보안 수정 기준 경고 → 6, 7. 거부 기준 2.38 → 8(D-036 확정 시) |
| CSP와 capability | 일부 됨 | CSP 있음: `src-tauri/tauri.conf.json:25`. `withGlobalTauri` 없음(기본 꺼짐): `tauri.conf.json:13-27`. `tauri` 기능 목록이 비어 릴리스에 개발자 도구 없음: `src-tauri/Cargo.toml:45`. `fs`·`shell` 플러그인 없음: `src-tauri/Cargo.toml:26-33`, `src-tauri/src/lib.rs:19` | CSP를 D-037 문자열과 맞추기, `core:default`가 넓음(`capabilities/default.json:6`), 앱 명령 허용 목록 없음(`build.rs:2`), 설정 검사 테스트 없음 → 9. 열린 저장소만 받기 → 10(제안) |
| innerHTML 금지 규칙 | 안 됨 | `eslint.config.js`에 규칙 없음. 지금 코드에 사용처는 없음 | → 11 |
| forbid unsafe | 안 됨 | 어느 crate 루트에도 없음. platform 테스트가 `unsafe { set_var }`를 씀: `crates/platform/src/app_dir.rs:33`, `:43`, `:45` | → 12. clippy `unwrap_used` 등 → 13(선택) |
| cargo-deny와 pnpm 고정 | 일부 됨 | `packageManager`: `package.json:6`. `pnpm-lock.yaml` 있음. CI `--frozen-lockfile`: `.github/workflows/ci.yml:58` | `deny.toml`과 CI 작업 → 14. 설치 스크립트 허용 목록 → 15 |
| Actions 권한 | 일부 됨 | `pull_request_target` 없음: `ci.yml:3-8` | `permissions` 없음, 액션이 태그로 고정됨(`ci.yml:27`, `:37`, `:39`, `:47`, `:115`) → 16 |
| code-analyst 점검표 | 됨 | `.claude/agents/code-analyst.md:34-42` | 줄 두 개 추가만(3, 9단계에서) |

참고로 D-038의 TS `strict`, `noUncheckedIndexedAccess`도 이미 됨(`tsconfig.json:8-9`).

## 완료 기준
- 저장소 안에 `refs/heads/-x`, `refs/heads/--output=…` 같은 ref가 있어도 hiGit의 어떤 git 호출도 그것을 옵션으로 읽지 않는다(git 크레이트 테스트).
- `ext::`, `file://`, 로컬 경로 원격으로 git을 부르면 "transport not allowed"로 실패하고 아무 명령도 실행되지 않는다. 테스트에서만 로컬 원격을 켤 수 있다.
- 저장소 설정의 `core.editor`, `core.pager`가 hiGit의 git 호출에서 실행되지 않는다.
- 상위 프로세스에 `GIT_DIR`이 걸려 있어도 hiGit은 연 저장소에서 git을 부른다.
- 비교 보기에서 저장소 밖을 가리키는 파일 링크와 폴더 링크(Windows는 정션 포함)가 막힌다. 안을 가리키는 링크는 내용 대신 "링크: 대상 경로"가 보인다.
- Windows에서 Git for Windows가 보안 기준보다 낮으면 시작 화면 첫 줄에 업데이트 안내와 내려받기 버튼이 보인다. Ubuntu 22.04 기본 git(2.34.1, 배포판 패키지)에서는 열린 질문 Q1의 답대로 동작한다.
- 화면 코드에 `dangerouslySetInnerHTML`, `innerHTML`, `outerHTML`, `insertAdjacentHTML`, `eval`, `new Function`을 쓰면 `pnpm lint`가 실패한다.
- 모든 Rust crate 루트에 `#![forbid(unsafe_code)]`가 있고, `cargo deny check`가 CI에서 매번 돈다.
- 워크플로 기본 권한이 `contents: read`이고 모든 액션이 커밋 SHA로 고정된다.
- 두 OS CI 초록. `pnpm tauri dev`와 릴리스 실행 파일에서 콘솔에 CSP 위반이 없고, 시작 화면·작업 공간·폴더 고르기·감시 이벤트가 그대로 동작한다(수동, 두 OS).

## 단계
각 단계 = 커밋 하나. Rust를 고친 단계는 커밋 전에 `pnpm check:full`. 바인딩이 바뀌는 단계는 같은 커밋에서 `cargo run -p app --bin gen-bindings`. 단계 번호는 이 계획 안의 번호이고, 001과의 관계는 단계마다 "001 관계"로 적는다. 권하는 순서는 맨 아래 "순서 요약".

### A. git 실행

1. git 실행 환경 보강 (`git: harden git child environment`)
   - 목표: D-035의 공통 환경을 다 채운다. 질문·편집기·페이저로 멈추지 않고, 위험한 전송 방식을 막는다.
   - 바꿀 파일: `crates/git/src/cli/runner.rs`, `crates/git/src/cli/mod.rs`(생성자), `crates/git/src/failure.rs`, `crates/git/src/cli/run_tests.rs` 또는 `mod.rs` 테스트, `docs/architecture.md` "GitBackend" 절의 환경 문단(85줄)과 "git 실행 규칙".
   - 내용:
     - `Runner::command`에 `PAGER=cat`, `GIT_EDITOR=:`, `GIT_ALLOW_PROTOCOL=https:ssh:git`을 더한다. `:`는 git이 편집기를 띄우지 않고 성공으로 보는 특별 값이라 Windows에서도 셸이 필요 없다(Q4).
     - 상위 프로세스에서 물려받으면 엉뚱한 저장소를 가리키는 변수를 지운다: `GIT_DIR`, `GIT_WORK_TREE`, `GIT_INDEX_FILE`, `GIT_OBJECT_DIRECTORY`, `GIT_ALTERNATE_OBJECT_DIRECTORIES`, `GIT_COMMON_DIR`, `GIT_NAMESPACE`. hiGit을 git hook이나 별칭 안에서 띄우면 생기는 일이다. D-035 목록 밖이지만 같은 취지라 넣는다(Q5).
     - 지금처럼 platform 설정이 먼저, 앱 고정 값이 나중에 이긴다.
     - 로컬 원격 테스트용: `CliBackend::allow_local_remotes_for_tests(self) -> Self`가 `GIT_ALLOW_PROTOCOL`을 `file:https:ssh:git`으로 바꾼다. 이름과 문서 주석에 테스트 전용임을 적는다. core 통합 테스트도 쓰므로 `pub`.
     - `failure.rs`에 `FailureKind::ProtocolNotAllowed`(stderr `transport '…' not allowed`). 화면 오류 코드와 문구는 001 10단계에서 다른 오류 코드와 함께 넣는다.
   - 테스트:
     - 기존 `every_call_disables_optional_locks`(`mod.rs:393`)처럼 별칭으로 자식 환경을 읽어 `PAGER`, `GIT_EDITOR`, `GIT_ALLOW_PROTOCOL` 값을 확인.
     - 실패해야 하는 경우: `ls-remote "ext::sh -c touch% <임시폴더>/pwned"` → `ProtocolNotAllowed`이고 `pwned` 파일이 없다. 임시 bare 저장소를 로컬 경로와 `file://`로 `ls-remote` → `ProtocolNotAllowed`. 같은 호출이 `allow_local_remotes_for_tests()`로는 성공.
     - 저장소 설정 `core.editor = "touch pwned"`, 스테이지한 파일이 있는 상태에서 메시지 없이 `commit`을 직접 실행 → 10초 안에 실패로 끝나고 `pwned`가 없다. `core.pager`도 같은 방식으로 `log`에서 실행되지 않음.
     - platform 설정 함수가 `GIT_DIR=<없는 곳>`을 넣어도 저장소에서 `rev-parse --show-toplevel`이 성공.
   - 완료 기준: `cargo test -p git` 두 OS 통과.
   - Windows/Ubuntu: 별칭 테스트는 Git for Windows의 sh를 쓴다(기존 테스트와 같음). `ext::` 명령 문자열의 공백 이스케이프(`% `)는 두 OS에서 같다. 로컬 경로 원격은 Windows에서 `C:\…` 그대로 쓴다.
   - 001 관계: **001 9단계(가져오기·올리기) 전에 해야 한다.** 9단계 테스트의 로컬 원격은 처음부터 `allow_local_remotes_for_tests()`를 쓴다. 6, 8단계와는 독립.

2. git 실행 파일을 절대 경로에서만 찾기 (`platform: resolve git only from absolute PATH entries`)
   - 목표: PATH에 `.`이나 상대 경로 항목이 있어도 저장소 폴더의 가짜 `git`을 고르지 않는다.
   - 바꿀 파일: `crates/platform/src/git_exe.rs`.
   - 내용: `find_git()`는 `std::env::split_paths(PATH)`에서 절대 경로 항목만 남겨 `which::which_in("git", Some(남긴 목록), 아무 절대 경로)`로 찾는다. 테스트하려고 순수 함수 `find_git_in(path_var: &OsStr) -> Option<PathBuf>`로 나누고 `find_git`은 이것을 부른다(환경 변수를 바꾸는 테스트는 `unsafe`라 12단계와 부딪힌다). 결과가 절대 경로가 아니면 버린다.
   - 테스트: 임시 폴더에 실행 가능한 가짜 `git`(Windows는 `git.exe`라는 빈 파일, Linux는 실행 비트를 준 sh 파일)을 두고 `find_git_in(".:<진짜 git 폴더>")`, `find_git_in("상대폴더;…")`가 진짜 git을 돌려준다. 절대 경로 항목만 있으면 그대로 찾는다.
   - 완료 기준: `cargo test -p platform` 두 OS 통과.
   - Windows/Ubuntu: PATH 구분자(`;`/`:`)는 `split_paths`/`join_paths`가 처리한다. Windows는 `PATHEXT`로 `git.cmd`·`git.bat`도 찾을 수 있다. Rust 1.77.2 이상이라 `.bat` 인자 문제는 없지만, `.exe`만 받을지는 Q6.
   - 001 관계: 독립.

3. 리비전 앞 `--end-of-options` (`git: put --end-of-options before revisions`)
   - 목표: 리비전·이름 인자가 `-`로 시작해도 git이 옵션으로 읽지 않는다.
   - 먼저 할 실험(커밋 안 함, 10분): 이 PC의 git 2.34.1에서 `git switch --end-of-options main`, `git checkout --end-of-options main`, `git reset --end-of-options HEAD`, `git rev-parse --verify --end-of-options HEAD`, `git branch --end-of-options x HEAD`, `git push --end-of-options origin HEAD:refs/heads/x`, `git log --end-of-options HEAD --`의 결과를 적는다. 연구 문서는 checkout·reset이 2.43.1부터 제대로 동작한다고 한다. 결과를 이 계획의 위험 절에 덧붙인다.
   - 바꿀 파일: `crates/git/src/cli/args.rs`(신규), `crates/git/src/cli/mod.rs`, `docs/architecture.md` "git 실행 규칙", `.claude/agents/code-analyst.md` 보안 점검 첫 줄.
   - 내용:
     - `args.rs`에 리비전 인자를 붙이는 한 함수를 둔다. 예: `fn push_revs(args: &mut Vec<&str>, cmd: Cmd, revs: &[&str], git: GitVersion)`. 명령별로 `--end-of-options`가 동작하는 최소 버전 표를 둔다(실험 결과로 채움. 예: switch·checkout·reset 2.43.1, rev-parse 2.30, 나머지 2.24). 그보다 낮으면 `--end-of-options`를 넣지 않는다. 이때는 4단계의 core 검사가 막는다.
     - `CliBackend`가 버전을 한 번 읽어 `OnceLock<GitVersion>`에 둔다(`version()` 결과 재사용).
     - 지금 호출에 적용: `log`의 리비전(`mod.rs:103`), `diff`의 기준(`mod.rs:143`), `rev-parse --verify -q HEAD^{commit}` 세 곳(`mod.rs:37`, `:91`, `:130`). 상수 인자에도 붙여 규칙을 "예외 없음"으로 둔다.
     - code-analyst 점검에 "버전 표에 따라 `--end-of-options`를 뺀 명령은 core 이름 검사를 거친 값만 받는다" 한 줄.
   - 테스트(실패해야 하는 경우 포함):
     - `update-ref refs/heads/-x HEAD`로 `-`로 시작하는 브랜치를 만들고(git이 거부하면 `.git/packed-refs`에 줄을 직접 써서 만든다. 받은 zip 저장소가 이런 모양일 수 있다) `log`에 `rev: Some("-x")` → 그 브랜치의 기록이 나온다(옵션 오류가 아님).
     - `log`에 `rev: Some("--output=<임시폴더>/pwned")` → 실패하고 `pwned` 파일이 생기지 않는다.
     - 버전 표 단위 테스트: 2.34.1에서 switch는 `--end-of-options`가 빠지고 log는 들어간다. 2.45.0에서는 둘 다 들어간다.
   - 완료 기준: `cargo test -p git` 두 OS 통과. `rg '"--end-of-options"' src-tauri/crates/git/src` 결과가 `args.rs`에만 있다.
   - Windows/Ubuntu: CI 두 OS의 git은 2.43.1보다 새것이라 낮은 버전 분기는 단위 테스트와 15b단계(최소 버전 CI)에서만 실제로 돈다.
   - 001 관계: **001 6, 8, 9단계 전에 해야 한다.** 6(브랜치 목록·이름 검사), 8(`git branch`, `git switch`), 9(`git push <원격> <refspec>`)는 처음부터 `args.rs`를 쓴다.

4. core: 옵션처럼 보이는 이름 막기 (`core: reject option-like names before git`)
   - 목표: 버전 때문에 `--end-of-options`를 못 쓰는 명령에서도 `-`로 시작하는 이름이 git에 닿지 않는다.
   - 바꿀 파일: `crates/core/src/actions/names.rs`(신규, `paths.rs` 옆), `crates/core/src/actions/mod.rs`.
   - 내용: `pub(crate) fn validate_ref_like(name: &str) -> Result<()>`. 빈 값, `-`로 시작, NUL·줄바꿈 포함이면 `InvalidInput`. 새 브랜치 이름은 001 6단계의 `check-ref-format` 검사를 따로 거치고, 이 함수는 저장소에 이미 있는 이름(바꿀 브랜치, 원격 이름, upstream)에 쓴다. 저장소에서 온 `-x` 브랜치는 목록에는 보이되 바꾸기를 누르면 "이 이름의 브랜치는 hiGit에서 다룰 수 없어요."가 뜬다(문구는 001 10단계 오류 문구와 함께).
   - 테스트: `-x`, `--upload-pack=x`, `-c`, 빈 값, 줄바꿈 → 거부. `main`, `기능/한글`, `a-b` → 통과.
   - 완료 기준: `cargo test -p higit-core`.
   - Windows/Ubuntu: 문자열 검사라 차이 없음.
   - 001 관계: 001 12(브랜치 만들기·바꾸기), 13(올리기의 원격·upstream) 전에. 001 10단계의 브랜치 이름 검사와 같은 파일에 둬도 된다(같이 해도 됨).

5. 링크를 따라가지 않는 비교와 경로 테스트 보강 (`core: show symlinks as link targets in diff`)
   - 목표: 비교 보기가 심볼릭 링크를 따라가 내용을 읽지 않는다. 밖을 가리키는 폴더 링크도 막힌다.
   - 바꿀 파일: `crates/core/src/query.rs`(`stays_inside` 옆), `crates/core/src/view/diff.rs`(`FileDiffView`에 `link: Option<String>`), `crates/platform/src/link.rs`(`symlink_dir` 추가, 테스트 도우미), 생성물, `src/features/diff/DiffView.tsx`, `src/messages/{ko,en}.ts`, `crates/core/tests/query.rs`.
   - 내용:
     - `Queries::diff`가 git을 부르기 전에 `symlink_metadata`로 본다. 링크면 git diff를 부르지 않고 `read_link` 결과를 `link: Some(대상)`으로 돌려준다. 밖을 가리키는지 여부와 상관없이 내용은 읽지 않는다. 지금처럼 밖을 가리키면 거부할지, 안·밖 모두 "링크: 대상"만 보여줄지는 Q7(추천: 둘 다 "링크: 대상"만, 내용을 안 읽으므로 거부할 이유가 없다).
     - 중간 폴더가 링크인 경로(`링크폴더/a.txt`)는 지금 `stays_inside`의 정규화로 막힌다. 테스트만 더한다.
     - 화면: `link`가 있으면 비교 칸에 "링크: {대상}" 한 줄(ko), "Link: {target}"(en). 대상 글자는 그대로 그린다.
   - 테스트(실패해야 하는 경우 포함):
     - 밖을 가리키는 폴더 링크 아래 파일(`링크폴더/비밀.txt`) → `InvalidInput`.
     - Windows: 정션(`mklink /J`가 권한 없이 됨)으로 같은 경우. 정션 생성은 platform 테스트 도우미에 둔다(OS별 코드).
     - 안을 가리키는 링크, 밖을 가리키는 링크 → `link: Some(…)`, git diff를 부르지 않음(가짜 백엔드 호출 기록이 비어 있음).
     - 경로 문자열: `../`, `/etc/passwd`, `C:/x`, `c:\x`, `\\server\share`는 이미 `paths.rs` 테스트에 있다. `Queries::diff`를 거친 통합 테스트에도 절대 경로와 드라이브 문자 두 줄을 더한다.
     - 화면: `link`가 있으면 "링크: …"가 보이고 줄 목록이 없다.
   - 완료 기준: `cargo test --workspace`, `pnpm test`, 바인딩 최신.
   - Windows/Ubuntu: Windows에서 파일·폴더 심볼릭 링크는 개발자 모드나 관리자 권한이 필요하다. 만들 수 없으면 기존 테스트처럼 건너뛰고(`tests/query.rs:395-398`) 정션 테스트는 늘 돈다. `core.symlinks=false`로 받은 저장소의 링크는 일반 파일이라 이 분기에 오지 않는다.
   - 001 관계: **001 10b단계(큰 새 파일 비교 상한) 전에, 또는 같이.** 10b는 git 대신 파일을 직접 읽으므로 같은 "링크는 따라가지 않음" 검사 뒤에 붙어야 한다. 10b의 직접 읽기는 이 단계의 `symlink_metadata` 결과를 받아서 쓴다.

### B. Git 버전

6. core·platform: 보안 수정 기준 판정 (`core: check git against security baseline`)
   - 목표: 거부 기준(`MIN_GIT_VERSION`)과 따로, 보안 수정이 들어간 버전인지 판정해 화면에 넘긴다. D-036 확정과 상관없이 할 수 있다(D-036이 말하는 "보안 수정 기준 경고" 부분만).
   - 바꿀 파일: `crates/core/src/git_security.rs`(신규), `crates/core/src/bootstrap.rs`, `crates/core/src/view/mod.rs`(`AppStatus`), `crates/platform/src/git_exe.rs`(`git_build`), `crates/platform/src/lib.rs`, `src-tauri/src/commands.rs`(`app_status`), 생성물, `src/messages/{ko,en}.ts`.
   - 내용:
     - core 상수 `GIT_SECURITY_BASELINE: &[GitVersion]`: 계열별로 보안 수정이 들어간 가장 낮은 버전. 연구 문서 기준으로는 CVE-2025-48384의 2.43.7, 2.44.4, 2.45.4, 2.46.4, 2.47.3, 2.48.2, 2.49.1, 2.50.1. 구현할 때 Git 보안 공지를 다시 확인해 그 뒤 공지가 있으면 반영한다. 판정: 같은 계열 항목이 있으면 그 이상인지, 표의 가장 낮은 계열보다 낮으면 낮음, 가장 높은 계열보다 높으면 괜찮음.
     - platform `git_build(git: &Path) -> GitBuild { Official, DistroPackage { version: String }, Unknown }`. Linux: 실제 경로를 풀어 `dpkg-query -S <경로>`가 성공하면 `dpkg-query -W -f '${Version}' git` 값을 담는다(`LC_ALL=C`, platform의 `configure_child` 적용). Windows: Git for Windows면 `Official`. 그 밖은 `Unknown`. OS 분기는 이 함수 안에만.
     - core 판정 결과 `GitSecurity { Ok, Outdated, DistroManaged }`. 배포판 패키지일 때 무엇을 돌려줄지는 Q1(추천: `DistroManaged`, 화면은 경고하지 않음).
     - `AppStatus`에 `git_security`, `min_git_version`(문자열), `git_package_version: Option<String>`. `gitTooOld` 문구의 숫자를 `min_git_version`에서 끼우게 바꿔(문구 함수에 인자) 8단계가 상수 한 줄만 바꾸게 한다. `CoreError::GitTooOld`일 때도 화면이 최소 버전을 알 수 있게 `ErrorView.detail`이나 별도 필드로 넘긴다.
     - Windows 버전 문자열 `2.45.1.windows.1`은 지금 파서가 `2.45.1`로 읽는다(`crates/git/src/types.rs:222`). 보안 수정이 `.windows.2`로만 나온 경우는 놓친다(위험에 적음).
   - 테스트: 판정 표(2.34.1 낮음, 2.43.6 낮음, 2.43.7 괜찮음, 2.50.0 낮음, 2.50.1 괜찮음, 2.60.0 괜찮음, 2.20.0 낮음). 배포판 패키지 + 2.34.1 → `DistroManaged`. platform: Ubuntu CI에서 `/usr/bin/git`이 dpkg 소유인지에 따라 결과를 출력만 하고(러너 git은 PPA라 버전이 다를 수 있음), `dpkg-query`가 없는 PATH로 부르면 `Unknown`.
   - 완료 기준: `cargo test --workspace`, `pnpm test`, 바인딩 최신.
   - Windows/Ubuntu: Ubuntu는 `git --version`이 배포판 패치 수준을 보여주지 않으므로(`git version 2.34.1`) 패키지 관리자에게 물어야 한다. `dpkg-query` 실행은 시작할 때 한 번, 수십 ms.
   - 001 관계: 독립. 003 시작 화면과 같은 `app_status`를 쓴다.

7. 화면: Git 업데이트 안내 (`screens: show git update notice on start screen`)
   - 목표: 보안 기준보다 낮으면 시작 화면 첫 줄에 "Git을 업데이트해 주세요"와 내려받기 버튼.
   - 바꿀 파일: `src/screens/Start/Start.tsx`와 테스트, `src/messages/{ko,en}.ts`, `src-tauri/Cargo.toml`(`tauri-plugin-opener`), `src-tauri/src/lib.rs`, `src-tauri/src/commands.rs`(`app_open_git_download`), `src-tauri/src/bindings.rs`, `crates/platform/src/git_exe.rs`(`git_download_url()`), `crates/core`(platform 값을 넘기는 얇은 함수), 생성물.
   - 내용:
     - 내려받기 주소는 platform이 정한다(Windows `https://git-scm.com/download/win`, Linux `https://git-scm.com/download/linux`). 화면은 주소를 넘기지 않고 `app_open_git_download` 명령만 부른다. Rust에서 opener 플러그인의 Rust API로 연다. 웹뷰에는 opener 권한을 주지 않는다(D-037 "https만 기본 브라우저"보다 더 좁다).
     - 문구(ko): "Git을 업데이트해 주세요. 보안 수정이 빠진 버전이에요." 버튼 "내려받기". en: "Please update Git. This version is missing security fixes." / "Download".
     - 작업 공간에도 보일지는 Q2. 앱을 켜면 마지막 프로젝트를 바로 열기 때문에(D-032) 시작 화면만으로는 거의 보이지 않는다.
     - 복제 전 한 번 더 확인(D-036)은 복제가 생기는 반복 4에서.
   - 테스트: `gitSecurity: "outdated"`면 안내와 버튼, `"ok"`·`"distroManaged"`면 없음. 버튼이 `api.openGitDownload`를 부름.
   - 완료 기준: `pnpm test`, `pnpm lint`, 바인딩 최신, 14단계가 먼저 됐다면 `cargo deny check` 통과(새 의존성). 수동: 두 OS에서 버튼이 기본 브라우저를 연다.
   - Windows/Ubuntu: Ubuntu는 `xdg-open`이 기본 브라우저를 고른다. 배포판 git이면 Q1 답에 따라 이 줄이 안 보일 수 있다.
   - 001 관계: 독립. 9단계(capability)가 먼저면 새 명령을 허용 목록에 함께 넣는다.

8. 거부 기준을 2.38로 (`core: raise minimum git version to 2.38`) — **D-036 확정 뒤에만**
   - 전제: 사람이 D-036을 확정한다. 확정하지 않으면 이 단계는 하지 않는다.
   - 바꿀 파일: `crates/core/src/bootstrap.rs:16`, `crates/git/src/cli/mod.rs:233`(버전 테스트), `docs/architecture.md:65`, `docs/decision.md`의 D-003 영향 문장은 고치지 않고 D-036 상태만 바꾼다, `packaging/windows/README.txt`·`README.md`의 "Git for Windows 2.30 이상".
   - 내용: `MIN_GIT_VERSION = 2.38.0`. 6단계에서 문구가 상수를 따르므로 문구 파일은 그대로다. 3단계의 버전 표에서 2.38보다 낮은 분기는 남겨 둔다(switch·checkout·reset은 여전히 2.43.1).
   - 테스트: 가짜 백엔드 2.37.9 → `GitTooOld`, 2.38.0 → 통과(`crates/core/tests/query.rs:29-34`를 고침).
   - 완료 기준: `cargo test --workspace`. 수동: Ubuntu 22.04 기본 git에서 앱이 "Git 2.38 이상이 필요해요"를 보인다(D-036 영향대로).
   - Windows/Ubuntu: Ubuntu 22.04 사용자는 PPA나 소스 설치가 필요해진다. 안내 문구에 Ubuntu 설치 방법을 넣을지는 Q3.
   - 001 관계: 001과 독립. 반복 2의 합치기 미리 계산(D-048) 전에 필요.

### C. 웹뷰

9. CSP와 capability 좁히기, 설정 검사 테스트 (`app: narrow csp and capabilities`)
   - 목표: D-037의 CSP와 "필요한 명령만" capability를 설정에 고정하고 테스트로 지킨다.
   - 바꿀 파일: `src-tauri/tauri.conf.json`, `src-tauri/capabilities/default.json`, `src-tauri/build.rs`, `src-tauri/tests/security_config.rs`(신규), `.claude/agents/code-analyst.md`, `docs/architecture.md` "보안" 절.
   - 내용:
     - CSP를 D-037 문자열로: `default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: asset:; connect-src ipc: http://ipc.localhost`. 지금 있는 `font-src 'self' data:`는 D-037에 없다. 글꼴이 파일로 번들되는지 확인하고 `data:`가 필요 없으면 지운다(`default-src 'self'`가 글꼴을 허용). `frame-src 'none'; object-src 'none'`을 더할지는 Q8.
     - `build.rs`: `tauri_build::try_build(Attributes::new().app_manifest(AppManifest::new().commands(&[…])))`로 앱 명령 목록을 선언한다. 이러면 capability에 없는 앱 명령은 웹뷰에서 부를 수 없다.
     - capability: `core:default` 대신 화면이 쓰는 것만. 지금 화면은 `@tauri-apps/api/event`의 `listen`(`src/lib/events.ts:2`), dialog `open`(`src/lib/dialog.ts:2`), 앱 명령만 쓴다 → `core:event:allow-listen`, `core:event:allow-unlisten`, `dialog:allow-open`, 앱 명령마다 `allow-<명령>`.
     - 테스트 `security_config.rs`: `tauri.conf.json`의 CSP가 기대 문자열과 같다, `unsafe-eval`·`http:`·`https:` 원격 출처가 없다, `withGlobalTauri`가 없거나 false다. capability에 `fs:`·`shell:`·`opener:` 권한과 `core:default`가 없다. `commands.rs`의 `#[tauri::command]` 함수 이름 목록과 capability의 `allow-*` 목록, `build.rs` 목록이 같다(파일을 읽어 정규식으로 비교).
     - code-analyst 점검에 "새 Tauri 명령은 `build.rs`와 capability에 함께 넣는다" 한 줄.
   - 테스트(수동): 두 OS에서 `pnpm tauri dev`와 `pnpm tauri build --no-bundle` 실행 파일로 시작 화면, 폴더 열기, 작업 공간, 파일 감시 이벤트, 비교 보기를 쓰고 웹 검사기 콘솔에 CSP·권한 오류가 없다. 릴리스 실행 파일에서 개발자 도구가 열리지 않는다.
   - 완료 기준: `cargo test --workspace`, 위 수동 확인.
   - Windows/Ubuntu: IPC 주소가 다르다. Windows(WebView2)는 `http://ipc.localhost`, Ubuntu(WebKitGTK)는 `ipc:`. 둘 다 CSP에 있어야 한다. 개발 모드는 Vite 서버를 쓰므로 CSP 적용 모양이 릴리스와 다를 수 있다. 판단은 릴리스 실행 파일로 한다.
   - 001 관계: **001 10단계(새 명령 `query_branches`, `query_branch_name`) 전에 하면** 새 명령을 처음부터 허용 목록에 넣는다. 뒤에 하면 그 명령까지 함께 넣는다. 19, 20단계는 새 명령이 없으면 영향 없음.

10. 열린 저장소만 받기 (`core: accept only opened repositories`) — 제안, Q9
    - 목표: 웹뷰가 임의 경로를 넘겨 git을 실행시키지 못하게 한다. 저장소 ID로 바꾸는 큰 작업(D-037 영향) 전의 가벼운 단계.
    - 까닭: 지금 `query_status(repo)` 등은 화면이 준 경로를 그대로 연다(`src-tauri/src/commands.rs:124-155`). 웹뷰에 스크립트가 끼어들면 받은 zip 폴더에서 `git status`를 돌려 그 저장소의 `core.fsmonitor`를 실행시킬 수 있다. D-039의 신뢰 확인을 `open_project`에만 넣으면 이 길로 우회된다.
    - 바꿀 파일: `crates/core/src/bootstrap.rs`(열린 경로 집합), `crates/core/src/query.rs`, `crates/core/src/watch.rs`, `crates/core/src/error.rs`, `crates/core/src/view/mod.rs`(`ErrorCode::NotOpened`), `src/messages/{ko,en}.ts`, 생성물, core 테스트.
    - 내용: `open_project`가 `Opened`나 `NotARepo`를 돌려줄 때 정규화한 경로를 `Core`의 집합에 넣는다. `Queries`의 모든 메서드, `preview`, `execute`, `watch`는 정규화한 경로가 집합에 없으면 `NotOpened`. `query_project_summary`는 최근 목록에 있는 경로도 받는다(시작 화면은 열지 않은 프로젝트의 상태를 읽는다). 명령 시그니처는 바뀌지 않는다.
    - 테스트: 열지 않은 저장소 경로로 `status` → `NotOpened`. 연 뒤에는 성공. 최근 목록에 없는 경로로 `project_summary` → `NotOpened`. init 흐름(저장소 아닌 폴더 열기 → init 실행)이 그대로 된다.
    - 완료 기준: `cargo test --workspace`, `pnpm test`, 바인딩 최신.
    - Windows/Ubuntu: 집합 비교는 `platform::same_path`(Windows 대소문자 무시).
    - 001 관계: 001 10단계의 새 조회도 같은 검사를 거친다(먼저 하면 저절로). "1b 끝나기 전"의 저장소 신뢰 확인보다 먼저 있어야 한다.

11. ESLint로 HTML 주입·eval 막기 (`lint: forbid innerHTML and eval in screens`)
    - 목표: 저장소에서 온 글자가 HTML로 해석되는 길을 린트로 닫는다.
    - 바꿀 파일: `eslint.config.js`, `scripts/eslint-rules.test.mjs`(신규).
    - 내용: `src/**/*.{ts,tsx}`에 `no-eval`, `no-implied-eval`, `no-new-func`와 `no-restricted-syntax` 선택자 `JSXAttribute[name.name='dangerouslySetInnerHTML']`, `MemberExpression[property.name=/^(innerHTML|outerHTML)$/]`, `CallExpression[callee.property.name='insertAdjacentHTML']`, `CallExpression[callee.object.name='document'][callee.property.name=/^write(ln)?$/]`. 메시지는 "저장소에서 온 글자는 React로 그대로 그립니다(D-037)." 마크다운이 필요해지면 DOMPurify를 거치는 부품 하나에만 예외를 둔다.
    - 테스트: ESLint Node API(`new ESLint().lintText(code, { filePath: "src/x.tsx" })`)로 금지 예시 여섯 개가 각각 오류이고, 평범한 JSX 글자는 오류가 없다.
    - 완료 기준: `pnpm lint`, `pnpm test`.
    - Windows/Ubuntu: 차이 없음.
    - 001 관계: 001 19, 20단계(화면) 전에 하면 새 화면 코드가 처음부터 검사받는다. 독립.

### D. 코드 규칙과 공급망

12. 모든 crate에 `forbid(unsafe_code)` (`chore: forbid unsafe code in all crates`)
    - 목표: D-038의 unsafe 금지를 컴파일러로 강제한다.
    - 바꿀 파일: `crates/{core,git,hosting,platform,watcher}/src/lib.rs`, `src-tauri/src/lib.rs`, `src-tauri/src/main.rs`(D-030의 `cfg_attr` 바로 다음 줄), `src-tauri/src/bin/gen-bindings.rs`, `crates/platform/src/app_dir.rs`, `scripts/rust-roots.test.mjs`(신규).
    - 내용:
      - 각 crate 루트 첫 속성으로 `#![forbid(unsafe_code)]`.
      - platform 테스트의 `unsafe { set_var }`를 없앤다: `app_data_dir`을 `app_data_dir_from(override: Option<OsString>, app_id)`로 나누고 테스트는 이 함수를 부른다. 그러면 platform도 `forbid`로 시작한다. 나중에 Windows API(Job Object, D-041)가 필요해지면 그때 `deny`로 바꾸고 쓰는 곳마다 이유 주석(D-038).
      - 통합 테스트 파일(`crates/*/tests/*.rs`)은 별도 crate라 위 속성이 닿지 않는다. 지금 unsafe가 없으니 code-analyst 점검으로 둔다.
    - 테스트: `scripts/rust-roots.test.mjs`가 `src-tauri/src/{lib,main}.rs`, `src-tauri/src/bin/*.rs`, `src-tauri/crates/*/src/lib.rs`를 읽어 모두 `#![forbid(unsafe_code)]`가 있는지 확인(새 crate를 만들고 빠뜨리면 실패). `cargo clippy`가 통과.
    - 완료 기준: `pnpm check:full` 통과. `rg "unsafe" src-tauri --glob '*.rs' --glob '!target/**'`가 비어 있다.
    - Windows/Ubuntu: 차이 없음. `app_data_dir_from`의 OS별 기본 위치는 지금처럼 `dirs`가 정한다.
    - 001 관계: 독립. 새 코드가 늘기 전에 일찍 하면 좋다.

13. clippy로 테스트 밖 `unwrap`·`expect`·`panic` 막기 (`chore: deny unwrap and panic outside tests`) — 선택, Q10
    - 목표: D-038의 "테스트 밖에서 경고 이상". "1b 지금" 목록에는 없지만 D-038 확정 사항이고, 지금 테스트 밖 사용처가 15곳 정도라 작다.
    - 바꿀 파일: `src-tauri/Cargo.toml`(`[workspace.lints.clippy] unwrap_used = "warn"`, `expect_used = "warn"`, `panic = "warn"`), 각 crate `Cargo.toml`(`[lints] workspace = true`), `src-tauri/clippy.toml`(`allow-unwrap-in-tests`, `allow-expect-in-tests`, `allow-panic-in-tests` = true), 사용처.
    - 내용: Mutex `lock().unwrap()`(`bootstrap.rs:42`, `:163`, `recent.rs:69`, `:81`, `:96`, `executor.rs:63`, `:119`, `watch.rs:50`, `:61`, `watcher/src/watch.rs:100`, `:125`)은 `unwrap_or_else(PoisonError::into_inner)`. `runner.rs:64`의 `expect`는 오류로 바꾼다. `src-tauri/src/lib.rs:23`(Tauri 실행 실패)와 `gen-bindings.rs`는 이유 주석과 함께 `#[allow]`. 통합 테스트 파일 맨 위에 `#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]`.
    - 테스트: `cargo clippy --workspace --all-targets -- -D warnings` 통과(CI가 `-D warnings`라 경고가 곧 실패).
    - 완료 기준: `pnpm check:full`.
    - Windows/Ubuntu: 차이 없음.
    - 001 관계: 독립. 하기로 하면 12단계 바로 뒤.

14. cargo-deny (`ci: check rust dependencies with cargo-deny`)
    - 목표: 취약점, 라이선스, 출처, 중복을 CI에서 매번 검사한다.
    - 바꿀 파일: `src-tauri/deny.toml`(신규), `.github/workflows/ci.yml`(작업 `deny`), `CLAUDE.md` "명령"(`cargo deny check`, src-tauri에서, 설치는 `cargo install cargo-deny --locked`).
    - 내용:
      - `[graph] targets = ["x86_64-unknown-linux-gnu", "x86_64-pc-windows-msvc"]`. 두 OS 의존성을 한 번에 본다. 그래서 작업은 ubuntu 하나로 충분하다.
      - `[advisories]`: 취약점은 실패. 관리 중단 경고는 직접 의존성만 실패로 보고(`unmaintained = "workspace"`), Tauri가 끌고 오는 GTK3 바인딩 등은 `ignore`에 RUSTSEC 번호와 이유("Tauri Linux 웹뷰 의존성. Tauri가 바꿀 때까지")를 적는다.
      - `[licenses]`: 허용 목록. 먼저 `cargo deny list`로 실제 목록을 뽑아 정한다. 예상: MIT, Apache-2.0, Apache-2.0 WITH LLVM-exception, BSD-2-Clause, BSD-3-Clause, ISC, Zlib, Unicode-3.0, MPL-2.0. 목록 밖은 crate별 `exceptions`에 이유와 함께.
      - `[bans] multiple-versions = "warn"`(Tauri 트리에 중복이 많아 실패로 두면 매번 깨진다), `wildcards = "deny"`. `[sources] unknown-registry = "deny"`, `unknown-git = "deny"`.
      - CI 작업은 `EmbarkStudios/cargo-deny-action`을 SHA로 고정(16단계 규칙).
    - 테스트: 로컬 `cargo deny check` 통과. 일부러 `deny.toml`에서 `MIT`를 빼면 실패하는지 한 번 확인(커밋 안 함).
    - 완료 기준: CI `deny` 작업 초록.
    - Windows/Ubuntu: 검사는 Ubuntu에서만 돌지만 `targets`로 Windows 전용 의존성(`windows-sys`, `webview2-com`)도 본다.
    - 001 관계: 독립. 새 의존성을 더하는 7단계(opener)보다 먼저 하면 그 의존성도 바로 검사받는다.

15. pnpm 설치 스크립트 제한 (`chore: allow only listed dependency build scripts`)
    - 목표: 의존성의 설치 스크립트는 적은 것만 돈다.
    - 바꿀 파일: `pnpm-workspace.yaml`(신규) 또는 `package.json`의 `pnpm` 필드, `pnpm-lock.yaml`(설정이 잠금 파일에 기록되면).
    - 내용: 고정한 pnpm 12.11.2에서 이 설정의 이름을 먼저 확인한다(pnpm 10은 `onlyBuiltDependencies`, 이후 버전에서 이름이 바뀌었을 수 있음). 지금 의존성 중 설치 스크립트가 필요한 것이 있는지 `pnpm install` 출력(무시된 빌드 목록)으로 확인한다. 없으면 빈 허용 목록, 있으면 그 패키지만 적는다. 허용되지 않은 빌드가 생기면 설치가 실패하게 하는 설정(pnpm 10의 `strictDepBuilds`에 해당)을 켠다.
    - 테스트: 새 clone에서 `pnpm install --frozen-lockfile` 성공, `pnpm tauri dev` 실행.
    - 완료 기준: 두 OS CI 초록.
    - Windows/Ubuntu: `@tauri-apps/cli`는 OS별 선택 의존성으로 바이너리를 받고 설치 스크립트를 쓰지 않는 것으로 안다. 두 OS에서 확인한다.
    - 001 관계: 독립.

15b. 최소 지원 git으로 git 크레이트 시험 (`ci: test git crate with ubuntu 22.04 distro git`) — 제안, Q11
    - 목표: GitHub 러너의 git은 최신이라 Ubuntu 22.04 기본 git(2.34.1)의 동작을 시험하지 못한다. 3단계 버전 표의 낮은 분기와 실패 분류 문구를 실제 낮은 버전으로 확인한다.
    - 바꿀 파일: `.github/workflows/ci.yml`(작업 `git-min`, `container: ubuntu:22.04`, `apt-get install git build-essential curl`, rustup, `cargo test -p git -p platform`).
    - 테스트: 이 작업에서 `git --version`이 2.34.1이고 git 크레이트 테스트가 통과.
    - 완료 기준: CI `git-min` 작업 초록.
    - Windows/Ubuntu: Ubuntu만. Windows의 오래된 Git for Windows 시험은 하지 않는다.
    - 001 관계: 3단계 뒤. 001 6, 8, 9단계의 테스트도 이 작업에서 돈다(낮은 버전에서 실패하면 그때 드러남).

16. GitHub Actions 권한과 SHA 고정 (`ci: pin actions by sha and restrict token permissions`)
    - 목표: 워크플로 토큰은 읽기만, 액션은 바뀌지 않는 커밋으로.
    - 바꿀 파일: `.github/workflows/ci.yml`.
    - 내용: 맨 위에 `permissions: contents: read`. 작업별로 더 필요한 권한은 없다(artifact 올리기는 `contents: read`로 된다). `actions/checkout`, `pnpm/action-setup`, `actions/setup-node`, `Swatinem/rust-cache`, `actions/upload-artifact`(14, 15b단계를 먼저 했다면 그 액션도)를 `uses: owner/repo@<40자 SHA> # v7`처럼 고정한다. SHA는 `gh api repos/<owner>/<repo>/git/ref/tags/<tag>`로 얻고, 주석 태그 객체면 한 번 더 풀어 커밋 SHA를 쓴다.
    - 테스트: push 후 두 OS CI 초록. `rg "uses: [^@]+@v" .github`가 비어 있다.
    - 완료 기준: 위 두 가지.
    - Windows/Ubuntu: 같은 파일.
    - 위험: 고정한 SHA는 저절로 올라가지 않는다. 의존성 업데이트 도구는 D-038대로 첫 배포 전에 정한다.
    - 001 관계: 독립.

17. 문서와 점검 (`docs: wrap up security 1b`) — 001 22단계와 합쳐도 됨
    - 바꿀 파일: `docs/architecture.md`("git 실행 규칙"에 `GIT_EDITOR=:`, 지우는 환경 변수, `args.rs`와 버전 표, 링크 규칙), `docs/roadmap.md`(1b에 "보안 1b, 계획 004"), `.claude/agents/code-analyst.md`(3, 9단계에서 더한 줄 확인).
    - code-analyst 점검: 3절 보안 점검 전부, `rg '"--end-of-options"'`가 `args.rs`에만, `rg "Command::new" src-tauri --glob '!**/tests/**'`가 `runner.rs`와 platform에만, capability에 `core:default` 없음.
    - 완료 기준: 위반 0건.

### 순서 요약
| 단계 | 001과의 관계 | 권하는 때 |
| --- | --- | --- |
| 12, 13, 11, 16, 15, 14 | 독립 | 맨 먼저. 도구 규칙이라 뒤에 오는 코드가 처음부터 검사받는다. 14는 7보다 먼저 |
| 1 | 001 9 **전** | 001 6 전 |
| 2 | 독립 | 1과 함께 |
| 3 | 001 6, 8, 9 **전** | 1 다음 |
| 4 | 001 12, 13 **전**, 001 10과 같이 해도 됨 | 3 다음 |
| 9 | 001 10 전이면 좋음 | 001 10 전 |
| 10 (제안) | 001 10 전이면 좋음, 신뢰 확인 **전** | 9 다음 |
| 5 | 001 10b **전** 또는 같이 | 001 10b 바로 앞 |
| 6, 7 | 독립 | 아무 때나. 7은 9, 14 뒤 |
| 15b (제안) | 3 뒤, 001 6·8·9 시험에 도움 | 3 다음 |
| 8 | D-036 확정에 달림 | 확정되면 |
| 17 | 001 22와 합침 | 마지막 |

## 1b 끝나기 전 항목 (이 계획에서는 단계로 쓰지 않음)
1b에 넣을지는 사용자에게 묻는다. 크기는 커밋 수 추정.

| 항목 | 결정 | 크기 | 먼저 있어야 하는 것 | 메모 |
| --- | --- | --- | --- | --- |
| 저장소 신뢰 확인 | D-039 | 중~대, 커밋 4~6 | 1(`GIT_EDITOR`가 `core.editor`를 이미 무력화), 10(없으면 조회 명령으로 우회), 003의 `Core::open_project`(있음, `bootstrap.rs:197` 자리) | git `config --local --list -z` 조회, 앱 데이터의 신뢰 기록(경로 + 설정 해시), 저장소별 `-c` 덮어쓰기를 `CliBackend`에 넘기는 방법(GitBackend 인자 변경), 빈 hooks 폴더(platform 앱 데이터), "믿고 열기·끄고 열기" 대화상자, dubious ownership과 `safe.directory` 한 경로 추가(전역 설정 쓰기 = 새 GitOp) |
| 비밀 파일 경고 | D-040 | 중, 커밋 3~5 | 001 11b(버전 저장, 있음) | core에 규칙 데이터(이름, 내용 패턴, 크기), 이번 버전에 넣은 파일 내용 읽기(`git cat-file --batch`), 알림 줄과 "이번 버전에서 빼기(있음)·제외할 파일에 추가(새 Action, 용어 확인)·그래도 저장". 저장을 멈추지 않는다(D-043) |
| 로그 가리기 | D-040 | 소~중, 커밋 2~3 | platform 앱 데이터 폴더(있음) | 지금 로그 기반이 없다(`eprintln!`만). 로그 파일·순환을 먼저 만들어야 한다. "자세히 보기"의 git 원문(`ErrorView.detail`)에도 주소 속 토큰이 나올 수 있어 001 9·13(올리기) 오류에 바로 쓸모가 있다 |
| 시간 제한과 취소 | D-041 | 대, 커밋 4~6 | 12(Windows Job Object는 unsafe·`windows-sys`가 필요해 platform을 `deny`로 바꿔야 함), 001 9·13 | `Runner`에 시간 제한과 프로세스 트리 종료(platform: Windows Job Object, Linux 프로세스 그룹), 네트워크 작업은 `--progress` stderr로 멈춤 감지, 취소 명령과 화면. 자동 원격 확인(D-045) 전에 필수 |
| index.lock 처리 | D-041 | 소~중, 커밋 2~3 | D-021 분류(있음) | `failure.rs`에 잠금 실패 분류, 살아 있는 git 프로세스 확인(platform, OS별), 파일 나이 확인, "멈춘 작업 정리" 확인 후 지우기 |

## 용어 변경
없음. 새 화면 문구(Git 업데이트 안내, "링크: …", 열지 않은 저장소 오류, 다룰 수 없는 브랜치 이름)는 `src/messages`에만 넣고 Git 용어가 아니다.

## 위험
- 3단계 실험 결과에 따라 버전 표가 바뀐다. 2.34.1에서 `switch --end-of-options`가 오류를 내면 그 명령은 core 검사(4단계)만이 방어선이다.
- `GIT_ALLOW_PROTOCOL`에 `http`가 없어 사내 http 서버를 쓰는 저장소는 가져오기·올리기가 막힌다(Q12).
- `GIT_EDITOR=:`는 편집기가 필요한 명령을 "기본 메시지로 성공"으로 만든다. hiGit은 메시지를 늘 `-F`나 `--no-edit`로 넘기므로 지금은 문제없다. 반복 2·3의 리베이스는 `GIT_SEQUENCE_EDITOR`를 따로 둔다.
- 9단계의 capability 축소는 런타임에만 드러나는 권한 오류를 낼 수 있다. 두 OS 릴리스 실행 파일로 화면을 한 바퀴 돈다.
- 6단계의 보안 기준 목록은 Git 보안 공지가 나올 때마다 손으로 올린다. Git for Windows가 `.windows.N`으로만 낸 보안 수정은 버전 파서가 놓친다.
- 배포판 판정은 dpkg 계열만 본다. Snap·Flatpak·직접 빌드한 git은 `Unknown`이라 공식 버전 기준으로 경고한다.
- 14단계의 라이선스 목록은 Tauri 버전을 올릴 때마다 바뀔 수 있다.

## 열린 질문
- Q1. Ubuntu 배포판 git(2.34.1-1ubuntu1.17처럼 버전 번호는 그대로 두고 보안 수정을 넣음)을 어떻게 다룰까요?
  - 가: 배포판 패키지면 경고하지 않는다(추천). 보안 수정은 OS 업데이트가 맡는다. 시작 화면 "Git을 찾았어요" 줄에 패키지 버전만 함께 보여준다.
  - 나: 경고하되 문구를 "시스템 업데이트로 Git 보안 수정을 받아 주세요"로 바꾼다. 늘 떠서 무시되기 쉽다.
  - 다: 배포판별 패치 패키지 버전 표를 core 상수로 두고 비교한다. 정확하지만 공지마다 Ubuntu·Debian 표도 올려야 한다.
  - 답은 D-036을 확정할 때 그 항목에 함께 적는다.
- Q2. Git 업데이트 안내를 시작 화면에만 둘까요, 작업 공간에도 한 번 보일까요? 앱을 켜면 마지막 프로젝트를 바로 열어서(D-032) 시작 화면만으로는 거의 보이지 않습니다. 추천: 작업 공간 위에 닫을 수 있는 안내 줄, Git 버전별로 한 번.
- Q3. D-036(거부 기준 2.38)을 확정할까요? 확정하면 Ubuntu 22.04 기본 git에서 앱이 켜지지 않습니다. 대안: 거부는 2.30으로 두고 2.38 미만이면 합치기 미리 계산만 끈다. 확정한다면 거부 화면에 Ubuntu용 설치 안내(git-core PPA)를 넣을까요?
- Q4. `GIT_EDITOR` 값을 `:`(편집기 없이 기본값으로 성공)로 할까요, `false`(편집기가 필요하면 실패)로 할까요? 추천은 `:`. git이 특별 값으로 처리해 Windows에서도 셸이 필요 없습니다.
- Q5. D-035 목록에 없는 `GIT_DIR`·`GIT_WORK_TREE` 등 저장소 위치 환경 변수를 지우는 것을 1단계에 넣을까요? 추천: 넣는다. 확정하면 D-035 영향에 한 줄 보충이 필요할 수 있다.
- Q6. Windows에서 git 실행 파일이 `.exe`가 아니면(`git.cmd`, `git.bat`) 거부할까요? 추천: 거부하지 않는다. Rust 1.77.2 이상이라 인자 문제는 없다.
- Q7. 비교 보기에서 밖을 가리키는 링크를 지금처럼 오류로 막을까요, 안을 가리키는 링크처럼 "링크: 대상"만 보여줄까요? 추천: 둘 다 "링크: 대상". 내용을 읽지 않으므로 막을 까닭이 없다.
- Q8. CSP에 D-037 문자열 밖의 `frame-src 'none'; object-src 'none'`을 더할까요? D-037의 "iframe을 쓰지 않는다"를 설정으로 옮기는 것입니다. 기존 `font-src 'self' data:`는 필요 없음을 확인하면 지워도 될까요?
- Q9. 10단계(열린 저장소만 받기)를 1b에 넣을까요? 저장소 신뢰 확인(D-039)을 1b에 넣는다면 사실상 필요합니다. 저장소 ID로 명령 시그니처를 바꾸는 일(D-037 영향)은 언제 할까요?
- Q10. 13단계(clippy `unwrap_used` 등)를 1b에 넣을까요? "1b 지금" 목록에는 없지만 D-038 확정 사항이고 작습니다.
- Q11. 15b단계(Ubuntu 22.04 컨테이너에서 git 크레이트 시험)를 넣을까요? CI 시간이 몇 분 늘어납니다.
- Q12. `GIT_ALLOW_PROTOCOL`에 `http`(암호화 없음)를 넣지 않는 D-035를 그대로 둘까요? 사내 http 서버 사용자는 막힙니다. 고급 설정으로 켜는 길은 로컬 경로 원격과 같이 나중에.
- Q13. "1b 끝나기 전" 다섯 항목 중 무엇을 1b에 넣을까요? 위 표의 크기와 의존성을 참고해 주세요.
