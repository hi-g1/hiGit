; hiGit NSIS 설치 파일 훅(D-220). 지울 때 앱이 만든 폴더 오른쪽 클릭 메뉴(HKCU, D-214)만 지운다.
; 앱은 설치 파일과 같은 사용자 범위(HKCU)에만 메뉴를 만든다. 다른 키는 건드리지 않는다.

!macro NSIS_HOOK_POSTUNINSTALL
  DeleteRegKey HKCU "Software\Classes\Directory\shell\higit"
  DeleteRegKey HKCU "Software\Classes\Directory\Background\shell\higit"
!macroend
