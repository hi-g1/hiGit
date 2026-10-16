Git을 모르는 사람도 쉽게 쓰는 데스크톱 Git 앱 hiGit의 시험판입니다.

## 설치

- **Windows 10·11 (64비트)**: `hiGit_*_x64-setup.exe`를 실행하세요. 관리자 확인 없이 지금 사용자에게 설치됩니다.
  서명하지 않은 시험판이라 "Windows의 PC 보호" 경고가 뜨면 "추가 정보" → "실행"을 누르세요.
  [Git for Windows](https://git-scm.com/download/win)가 필요합니다.
- **Ubuntu 22.04 이상**: `sudo apt install ./hiGit_*_amd64.deb` (Git도 함께 설치됩니다).
  AppImage는 `libfuse2`와 Git이 필요합니다.
- 사용법은 `hiGit-guide.html`을 브라우저로 여세요.
- 받은 파일은 `SHA256SUMS`로 확인할 수 있습니다(`packaging/README.md`).

## 알려진 한계

- 코드 서명이 없습니다.
- GitHub 연결은 이 빌드에 client id가 없으면 "준비 안 됨"으로 보입니다.

문제나 제안은 Issues에, 보안 문제는 SECURITY.md의 비공개 신고로 알려 주세요.
