# hiGit

Git을 모르는 사람도 쉽게 쓰는 데스크톱 Git 앱. Tauri 2(Rust) + React/TypeScript. Windows와 Ubuntu 지원.

## 먼저 읽을 문서
- `docs/spec.md` — 무엇을 만드나, 화면, 기능과 용어표(용어의 원본)
- `docs/architecture.md` — 계층, 인터페이스, 새 액션 추가 절차
- `docs/decision.md` — 확정된 결정. 여기에 반하는 구현은 하지 않는다
- `docs/design.md` — 색, 글꼴, 부품 규칙
- `docs/roadmap.md` — 반복 순서, `docs/plans/` — 반복별 상세 계획
- `docs/research/` — 조사 결과. **결정이 아니라 참고 자료다.** 조사 내용을 근거로 구현하려면 먼저 `docs/decision.md`에 결정으로 올린다

## 작업 흐름
1. 새 기능이나 큰 변경은 `planner` 서브에이전트에게 계획을 맡긴다. 결과는 `docs/plans/NNN-<주제>.md`.
2. 사람이 계획을 확인한 뒤 구현한다.
3. 구현 전에 영향 범위가 불분명하면 `code-analyst` 서브에이전트에게 분석을 맡긴다.
4. 구현 후 `code-analyst`로 계층 규칙·용어 규칙 위반을 점검한다.
5. 결정이 바뀌면 `docs/decision.md`를 먼저 고치고 코드를 바꾼다.

## 반드시 지킬 규칙
- 화면 코드는 git을 직접 부르지 않는다. 상태를 바꾸는 기능은 `crates/core`의 `Action`, 조회는 `core::query`로 만들고 화면은 둘 다 core를 통해서만 부른다(D-010).
- 모든 git 호출은 `crates/git`의 `GitBackend`를 통한다. 출력은 `--porcelain`/`-z` 같은 기계용 형식만 파싱한다(D-003).
- 크레이트 의존 방향: `app → core → (git, hosting, platform, watcher)`. 아래 계층끼리 서로 의존하지 않는다.
- 화면에 Git 용어를 쓸 때는 항상 `<Term id="…" />`. 새 용어는 `docs/spec.md` 표와 `src/glossary/ko.ts`, `en.ts`를 함께 고친다(D-006, D-015).
- 위험도 `caution` 이상 액션은 실행기의 미리보기·백업·되돌리기 흐름을 우회하지 않는다(D-007).
- 색·간격·글꼴은 `src/styles/tokens.css` 변수만 쓴다. 색 값을 직접 쓰지 않는다.
- OS별 코드는 `crates/platform`에만 둔다.
- 사용자에게 보이는 문구는 한국어 존댓말, 짧게. Git 원문 오류는 "자세히 보기"로 접는다.

## 명령
저장소 루트에서 실행한다. `src-tauri/`에서 실행하는 것은 따로 적었다.
- 개발 실행: `pnpm tauri dev`
- 프론트 테스트: `pnpm test`
- 프론트 타입 검사: `pnpm typecheck`
- 프론트 린트: `pnpm lint` (ESLint + Stylelint. 화면의 `invoke` 직접 사용, tokens.css 밖의 색 값을 막는다)
- Rust 테스트: `cargo test --workspace` (src-tauri에서)
- Rust 린트: `cargo clippy --workspace --all-targets -- -D warnings` (src-tauri에서)
- Rust 형식: `cargo fmt --all --check` (src-tauri에서)
- 타입 바인딩 생성: `cargo run -p app --bin gen-bindings` (src-tauri에서). `src/lib/bindings.ts`, `src/lib/action-meta.json`을 다시 만든다. `core::view`나 Tauri 명령을 바꾸면 실행하고 결과를 함께 커밋한다(CI가 최신인지 확인).
- 빌드 확인: `pnpm tauri build --no-bundle`
- CI: `.github/workflows/ci.yml`(ubuntu-22.04, windows-latest). 결과는 `gh run list`.

## 커밋
- 한 커밋에 한 가지. 메시지는 `<영역>: <요약>` (예: `core: add amend action`).
- 테스트가 실패하는 상태로 커밋하지 않는다.
