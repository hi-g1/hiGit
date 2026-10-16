# 설치 파일 만들기와 확인

## 만들기

릴리스 설치 파일은 GitHub Actions에서만 만듭니다(D-038). 태그 `v<버전>`을 올리면 `.github/workflows/release.yml`이
Windows(NSIS 사용자 단위 설치 파일)와 Ubuntu(deb, AppImage)를 만들어 `SHA256SUMS`와 함께 시험판 릴리스로 올립니다.
수동 실행(workflow_dispatch)은 초안만 만듭니다.

로컬에서 시험으로 만들 때(저장소 루트):

```
pnpm tauri build --bundles nsis          # Windows
pnpm tauri build --bundles deb,appimage  # Ubuntu
```

태그와 버전은 `package.json`·`src-tauri/tauri.conf.json`·`src-tauri/Cargo.toml`이 같아야 합니다
(`node scripts/release-version.mjs v0.1.0`으로 확인).

## 받은 설치 파일 확인

Ubuntu:

```
sha256sum -c SHA256SUMS --ignore-missing
gpg --verify SHA256SUMS.asc SHA256SUMS   # 서명이 있을 때
```

Windows PowerShell:

```
Get-FileHash -Algorithm SHA256 .\hiGit_0.1.0_x64-setup.exe
```

나온 값을 `SHA256SUMS`의 같은 파일 줄과 비교하세요.

## 서명 비밀(정식 배포 전)

지금은 서명 자리만 있습니다(D-221). 비밀은 GitHub 저장소 설정 → Environments → `release`에 넣습니다.
비밀이 없으면 워크플로는 서명을 건너뛰고 릴리스 이름에 "서명 없는 시험판"을 붙입니다.

| 비밀 | 쓰는 곳 | 형태 |
| --- | --- | --- |
| `WINDOWS_SIGN_COMMAND` | Windows 설치 파일과 실행 파일 서명 | Tauri `bundle.windows.signCommand`. 서명할 파일 자리는 `%1` |
| `LINUX_GPG_KEY` | `SHA256SUMS.asc` | ASCII로 내보낸 GPG 비밀 키(암호 없음) |
| `HIGIT_GITHUB_CLIENT_ID` | GitHub 연결(D-109) | GitHub OAuth App의 공개 client id. 비밀이 아니라 저장소 변수(Settings → Secrets and variables → Actions → Variables) |

`WINDOWS_SIGN_COMMAND` 예시(형태만, 실제 값은 비밀에만):

```
signtool sign /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 /sha1 <인증서 지문> %1
trusted-signing-cli -e https://eus.codesigning.azure.net -a <계정> -c <인증서 프로필> %1
```

서명 명령은 저장소에 쓰지 않습니다. 워크플로가 비밀에서 `src-tauri/tauri.sign.json`을 잠깐 만들어 넘기고 지웁니다
(`.gitignore`에 있음).
