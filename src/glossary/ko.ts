import type { Glossary } from "./types";

// docs/spec.md "기능 목록과 용어표"를 그대로 옮긴 데이터. 표가 원본이다(D-006, D-024).
// spec-sync.test.ts가 두 곳이 같은지 확인한다.
export const ko: Glossary = {
  status: {
    friendly: "변경 사항",
    gitName: "status",
    command: "git status",
    description: "마지막 버전 이후 바뀐 파일 목록",
  },
  diff: {
    friendly: "비교 보기",
    gitName: "diff",
    command: "git diff",
    description: "파일이 어떻게 바뀌었는지 줄 단위로 비교",
  },
  stage: {
    friendly: "이번 버전에 넣기",
    gitName: "stage",
    command: "git add",
    description: "이번 버전에 포함할 파일 고르기",
  },
  unstage: {
    friendly: "이번 버전에서 빼기",
    gitName: "unstage",
    command: "git restore --staged",
    description: "이번 버전에 넣었던 파일을 다시 빼기",
  },
  commit: {
    friendly: "버전 저장",
    gitName: "commit",
    command: "git commit",
    description: "지금 상태를 설명과 함께 버전으로 남기기",
  },
  fetch: {
    friendly: "원격 확인",
    gitName: "fetch",
    command: "git fetch",
    description: "다른 사람이 올린 버전이 있는지 확인하기. 내 파일은 바뀌지 않아요.",
  },
  pull: {
    friendly: "가져오기",
    gitName: "pull",
    command: "git pull --ff-only",
    description: "다른 사람이 올린 버전을 받아 지금 브랜치에 합치기",
  },
  push: {
    friendly: "올리기",
    gitName: "push",
    command: "git push",
    description: "내 버전들을 원격 저장소로 보내기",
  },
  sync: {
    friendly: "동기화",
    gitName: "sync",
    command: "git pull + git push",
    description: "원격 저장소의 새 버전을 가져온 다음 내 버전을 올리기",
  },
  branch: {
    friendly: "브랜치",
    gitName: "branch",
    command: "git branch",
    description: "원래 작업에 영향 없이 따로 작업하는 공간",
  },
  "branch-delete": {
    friendly: "브랜치 지우기",
    gitName: "branch -D",
    command: "git branch -D",
    description: "브랜치 이름표를 지우기. 버전은 작업 내역에 남아요.",
  },
  "branch-rename": {
    friendly: "브랜치 이름 바꾸기",
    gitName: "branch -m",
    command: "git branch -m",
    description: "브랜치 이름을 바꾸기. 원격의 같은 이름은 그대로예요.",
  },  "remote-branch-delete": {
    friendly: "원격 브랜치 지우기",
    gitName: "push --delete",
    command: "git push <원격> --delete <브랜치>",
    description: "원격 저장소의 브랜치를 지우기. 다른 사람의 목록에서도 사라져요.",
  },
  "remote-branch-rename": {
    friendly: "원격 브랜치 이름 바꾸기",
    gitName: "push + push --delete",
    command: null,
    description: "원격 저장소에 새 이름으로 올리고 옛 이름을 지우기. 다른 사람이 쓰던 옛 이름은 끊겨요.",
  },

  switch: {
    friendly: "브랜치 바꾸기",
    gitName: "switch",
    command: "git switch",
    description: "다른 브랜치로 옮겨 가기",
  },
  log: {
    friendly: "버전 기록",
    gitName: "log",
    command: "git log",
    description: "지금까지 저장한 버전 목록",
  },
  merge: {
    friendly: "합치기",
    gitName: "merge",
    command: "git merge",
    description: "다른 브랜치의 작업을 지금 브랜치에 합치기. 병합이라고도 해요.",
  },
  conflict: {
    friendly: "충돌",
    gitName: "merge conflict",
    command: null,
    description: "두 브랜치가 같은 곳을 다르게 고쳐서 어느 쪽을 남길지 정해야 하는 상태",
  },
  ours: {
    friendly: "내 브랜치",
    gitName: "ours",
    command: null,
    description: "합치는 중 지금 브랜치 쪽 내용",
  },
  theirs: {
    friendly: "합칠 브랜치",
    gitName: "theirs",
    command: null,
    description: "합치는 중 가져오는 쪽 내용",
  },
  base: {
    friendly: "고치기 전 원래 코드",
    gitName: "base",
    command: null,
    description: "두 브랜치가 갈라지기 전의 코드",
  },
  stash: {
    friendly: "임시 보관",
    gitName: "stash",
    command: "git stash",
    description: "하던 작업을 잠시 보관했다가 나중에 꺼내기",
  },
  park: {
    friendly: "여기 두기",
    gitName: "stash",
    command: "git stash -u",
    description: "브랜치를 바꿀 때 고친 파일을 그 브랜치에 두었다가 돌아오면 꺼내기",
  },
  discard: {
    friendly: "고친 것 버리기",
    gitName: "restore",
    command: "git restore",
    description: "아직 버전으로 저장하지 않은 변경을 버리고 마지막 버전으로 돌아가기",
  },
  pr: {
    friendly: "검토 요청",
    gitName: "Pull Request",
    command: null,
    description: "내 브랜치를 합쳐 달라고 팀에 요청",
  },
  "pr-draft": {
    friendly: "초안",
    gitName: "draft pull request",
    command: null,
    description: "아직 검토를 받을 준비가 안 된 검토 요청. 합칠 수 없어요.",
  },
  "pr-merge": {
    friendly: "검토 요청 합치기",
    gitName: "merge pull request",
    command: null,
    description: "검토 요청의 브랜치를 GitHub에서 대상 브랜치에 합치기. 앱에서 되돌릴 수 없어요.",
  },
  "pr-close": {
    friendly: "검토 요청 닫기",
    gitName: "close pull request",
    command: null,
    description: "합치지 않고 검토 요청을 닫기. 다시 열 수 있어요.",
  },
  checks: {
    friendly: "시험 결과",
    gitName: "checks",
    command: null,
    description: "올린 버전에 자동으로 돌린 시험과 검사의 결과",
  },  review: {
    friendly: "리뷰",
    gitName: "review",
    command: null,
    description: "검토 요청에 팀원이 남기는 승인이나 변경 요청",
  },
  comment: {
    friendly: "댓글",
    gitName: "comment",
    command: null,
    description: "검토 요청에 남기는 글. 저장소를 볼 수 있는 사람이 모두 봐요.",
  },

  clone: {
    friendly: "복제",
    gitName: "clone",
    command: "git clone",
    description: "원격 저장소를 내 컴퓨터로 복사",
  },
  fork: {
    friendly: "포크",
    gitName: "fork",
    command: null,
    description: "내 GitHub 계정에 복사본 만들기. 그 복사본에 올리고 원래 저장소에 검토 요청을 보내요.",
  },  publish: {
    friendly: "GitHub에 올리기",
    gitName: "new repository",
    command: "git push -u",
    description: "이 프로젝트로 GitHub에 새 저장소를 만들고 올리기. 만든 저장소는 앱에서 지울 수 없어요.",
  },

  init: {
    friendly: "버전 관리 시작",
    gitName: "init",
    command: "git init",
    description: "이 폴더를 Git으로 버전 관리하기 시작",
  },  identity: {
    friendly: "이름과 이메일",
    gitName: "user.name, user.email",
    command: "git config --global user.name",
    description: "저장한 버전마다 남는 이름과 이메일",
  },

  amend: {
    friendly: "마지막 버전 고치기",
    gitName: "amend",
    command: "git commit --amend",
    description: "마지막 버전의 설명이나 내용 고치기",
  },
  revert: {
    friendly: "되돌리기",
    gitName: "revert",
    command: "git revert",
    description: "고른 버전의 변경을 취소하는 새 버전 만들기. 기록은 그대로 남아요.",
  },
  reset: {
    friendly: "이 버전으로 돌아가기",
    gitName: "reset",
    command: "git reset",
    description: "브랜치를 고른 버전으로 옮기기. 그 뒤의 버전은 기록에서 빠져요.",
  },
  "rebase-i": {
    friendly: "기록 정리",
    gitName: "interactive rebase",
    command: "git rebase -i",
    description: "버전 순서 바꾸기, 묶기, 메시지 고치기",
  },
  rebase: {
    friendly: "리베이스",
    gitName: "rebase",
    command: "git rebase",
    description: "내 브랜치의 버전들을 다른 브랜치의 최신 버전 뒤로 옮기기",
  },
  "cherry-pick": {
    friendly: "이 버전만 복사하기",
    gitName: "cherry-pick",
    command: "git cherry-pick",
    description: "다른 브랜치의 버전 하나를 지금 브랜치로 복사하기",
  },
  tag: {
    friendly: "태그",
    gitName: "tag",
    command: "git tag",
    description: "이 버전에 v1.0 같은 이름 붙이기",
  },  "tag-push": {
    friendly: "태그 올리기",
    gitName: "push tag",
    command: "git push <원격> <태그>",
    description: "이 태그를 원격 저장소로 보내기. 같은 이름의 태그를 덮어쓰지 않아요.",
  },
  "remote-tag-delete": {
    friendly: "원격 태그 지우기",
    gitName: "push --delete tag",
    command: "git push <원격> --delete <태그>",
    description: "원격 저장소의 태그를 지우기. 내 컴퓨터의 태그는 그대로예요.",
  },

  blame: {
    friendly: "줄별 기록",
    gitName: "blame",
    command: "git blame",
    description: "각 줄을 마지막으로 바꾼 사람과 날짜",
  },
  reflog: {
    friendly: "작업 내역",
    gitName: "reflog",
    command: "git reflog",
    description: "최근 작업을 되짚어 지운 버전이나 브랜치를 되살리기",
  },
  "force-push": {
    friendly: "덮어쓰며 올리기",
    gitName: "force push",
    command: "git push --force-with-lease",
    description: "원격 저장소의 기록을 내 기록으로 덮어쓰기. 다른 사람의 작업이 사라질 수 있어요.",
  },
  remote: {
    friendly: "원격 저장소",
    gitName: "remote",
    command: "git remote",
    description: "이 프로젝트가 연결된 원격 저장소 목록",
  },
  ignore: {
    friendly: "제외할 파일",
    gitName: ".gitignore",
    command: null,
    description: "버전에 넣지 않을 파일 규칙",
  },
  lock: {
    friendly: "멈춘 작업 정리",
    gitName: "index.lock",
    command: null,
    description: "중간에 끊긴 Git 작업이 남긴 잠금 파일 지우기. 실행 중인 Git이 없을 때만 해요.",
  },
};
