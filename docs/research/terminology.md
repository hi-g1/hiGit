# 화면 용어 조사: 억지 번역 대신 이미 아는 말

- 날짜: 2026-10-15
- 상태: 조사 결과. 채택하면 `docs/decision.md`에 결정으로 올리고 `docs/spec.md` 용어표와 glossary `ko.ts`, 화면 문구를 함께 고친다.
- 시안 페이지: https://claude.ai/artifact/5Ezm6FAaCJtJZn4c4Upebm 의 "용어" 부분

## 문제
지금 용어표는 모든 Git 용어를 우리말로 바꾸다 보니 "작업 갈래", "갈래 옮겨 심기", "새 소식 확인", "안전망"처럼 hiGit에서만 쓰는 말이 생겼다. 밖에서 Git을 배울 때 이어지지 않고, 처음 보는 사람도 은유를 한 번 더 풀어야 한다.
한국어판 도구도 같은 문제를 겪는다. GitHub 한국어 문서는 branch를 "분기", pull request를 "끌어오기 요청", main을 "주 분기"로 옮겼고, 같은 문서 안에 "브랜치 생성하기"와 "머지"가 섞여 있다.

## 기준
1. 구글 문서, 드롭박스, 메일처럼 이미 쓰는 앱에 익숙한 말이 있으면 그 말을 쓴다. 버전 기록, 변경 사항, 임시 보관, 복구, 동기화.
2. 한국어로 굳은 외래어는 번역하지 않는다. 브랜치, 태그, 리베이스는 그대로 쓰고 툴팁으로 뜻을 한 줄 보여준다.
3. 은유 대신 하는 일을 말한다. "안전망"보다 "복구", "새 소식 확인"보다 "새로 고침".
4. 한 낱말은 한 기능에만 쓴다. "가져오기"는 pull에만, clone은 "복제".
5. 브랜치 이름 같은 고유 이름은 번역하지 않는다. main은 화면에서도 main.
6. 괄호로 덧붙이는 설명을 쓰지 않는다. 보충은 툴팁이나 둘째 줄로.

## 바꿀 안
| Git | 지금 | 바꿀 안 | 이유 |
| --- | --- | --- | --- |
| branch | 작업 갈래 | 브랜치 | VS Code, Google Cloud 한국어판, 강의가 모두 브랜치. 갈래는 hiGit 밖에서 통하지 않음 |
| commit | 저장하기 | 버전 저장 | 파일 저장 Ctrl+S와 헷갈림. 구글 문서의 "버전"으로 구분 |
| log | 저장 기록 | 버전 기록 | 구글 문서 메뉴 이름과 같음 |
| status | 바뀐 내용 | 변경 사항 | 구글 문서, VS Code 한국어판 |
| stage | 저장에 넣기 | 이번 버전에 넣기 | 파일 옆 체크 상자로 보여주고 낱말은 툴팁에만 |
| fetch | 새 소식 확인 | 새로 고침 | 원격 상태를 다시 읽는 일 |
| pull | 최신으로 받기 | 가져오기 | |
| push | 올리기 | 올리기 | 유지 |
| pull + push | 없음 | 동기화 | Google Cloud 한국어판 "변경사항 동기화" |
| merge | 합치기 | 합치기 | 유지. 툴팁에 병합, merge |
| conflict | 겹친 부분 고르기 | 충돌, 버튼은 충돌 해결 | 드롭박스 "충돌된 사본"으로 익숙 |
| ours · theirs | 내 갈래 · 가져오는 갈래 | 내 브랜치 · 합칠 브랜치 | VS Code의 "현재 변경 사항 · 수신 변경 사항"은 직역이라 어느 쪽인지 안 보임 |
| stash | 잠시 치워두기 | 임시 보관 | 메일 임시 보관함 |
| reflog | 안전망 | 복구 | 은유 대신 하는 일 |
| rebase | 갈래 옮겨 심기 | 리베이스 | 가끔 쓰는 고급 기능은 원어 + 툴팁 |
| interactive rebase | 기록 정리하기 | 기록 정리 | 유지 |
| tag | 버전 표시 | 태그 | 짧고 익숙함 |
| clone | 프로젝트 가져오기 | 복제 | pull과 겹치지 않게 |
| init | 관리 시작 | 버전 관리 시작 | |
| remote | 연결된 저장소 | 원격 저장소 | 도움말·강의 표준 |
| reset | 기록째 되돌리기 | 이 버전으로 되돌리기 | 위험 표시와 함께 |
| revert | 기록 남기고 취소 | 되돌리기 | 기록이 남는 안전한 쪽을 기본 되돌리기로 |
| amend | 방금 저장 고치기 | 마지막 버전 고치기 | commit 용어에 맞춤 |
| cherry-pick | 저장 하나만 가져오기 | 이 버전만 가져오기 | commit 용어에 맞춤 |
| blame | 누가 언제 바꿨나 | 줄별 기록 | 질문형 메뉴 이름은 다른 메뉴와 어울리지 않음 |
| discard | 변경 취소 | 변경 취소 | 유지 |
| force push | 강제로 올리기 | 덮어쓰며 올리기 | 무슨 일이 일어나는지 말함 |
| .gitignore | 저장 안 할 파일 | 제외할 파일 | 버전 저장과 섞이지 않게 |
| pull request | 검토 요청 | 검토 요청 | 유지. "끌어오기 요청"은 피함 |

## 반영할 곳
- `docs/spec.md` 용어표와 화면 목록의 이름
- glossary `ko.ts`, `en.ts`. spec 표와 맞추는 테스트가 있으니 함께 고친다
- 이미 만든 화면 문구와 Action 이름표
- `docs/research/git-visualization.md`, `design-direction.md`의 "갈래" 표기

## 출처
- GitHub 한국어 문서 Hello World: https://docs.github.com/ko/enterprise-server@3.15/get-started/start-your-journey/hello-world
- Google Cloud Shell 편집기 버전 제어 한국어: https://docs.cloud.google.com/shell/docs/version-control?hl=ko
- Google 한국 블로그, 문서 버전 기록과 수정 사항 제안: https://korea.googleblog.com/2017/08
