hiGit 시험판 (Ubuntu 22.04 이상, x64)
====================================

설치(deb, 추천)
  sudo apt install ./hiGit_*_amd64.deb
  Git도 함께 설치됩니다. 지울 때는 sudo apt remove hi-git

AppImage(설치 없이)
  chmod +x hiGit_*_amd64.AppImage 로 실행 권한을 준 뒤 더블클릭하세요.
  Ubuntu 22.04에서는 libfuse2가 필요합니다: sudo apt install libfuse2
  Git은 따로 설치하세요: sudo apt install git

폴더 오른쪽 클릭 메뉴
  처음 켜면 파일 앱 오른쪽 클릭의 "스크립트"에 "Git으로 관리"가 생깁니다.
  설정 "일반"의 "폴더 오른쪽 클릭 메뉴"에서 끌 수 있습니다.

라이선스
  앱은 MIT 라이선스입니다. 함께 들어 있는 글꼴과 라이브러리의 라이선스는
  THIRD_PARTY_NOTICES.md에 있습니다(deb는 /usr/share/doc/higit/).

문제가 있으면 저장소의 Issues에 알려 주세요.
