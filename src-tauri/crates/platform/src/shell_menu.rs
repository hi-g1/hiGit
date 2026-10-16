//! 폴더 오른쪽 클릭 메뉴 "hiGit으로 열기"(D-214). Windows는 탐색기(사용자 레지스트리 HKCU), Ubuntu는 Nautilus
//! 스크립트. 관리자 권한 없이 만들고 지운다.
//!
//! 메뉴가 넘기는 명령은 `"<실행 파일>" -- "<폴더>"`다. `--` 뒤라 `-`로 시작하는 폴더 이름도 옵션으로 읽히지 않는다.
//! 등록에 들어가는 글자는 실행 파일 경로와 항목 이름뿐이다.

use std::io;
use std::path::{Path, PathBuf};

/// 이 OS에서 쓰는 메뉴 종류. 화면은 이것으로 설명 문구를 고른다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellMenuKind {
    /// Windows 탐색기. Windows 11은 "더 많은 옵션 표시" 안에 나온다.
    Explorer,
    /// Ubuntu Nautilus의 "스크립트" 메뉴.
    NautilusScripts,
    Unsupported,
}

/// 지금 등록 상태.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellMenuState {
    /// 이 실행 파일과 이 이름으로 등록되어 있다.
    Installed,
    /// hiGit이 등록했지만 실행 파일이나 이름이 다르다(앱을 옮겼거나 언어를 바꿨다). 다시 등록하면 고쳐진다.
    Outdated,
    NotInstalled,
    /// 같은 이름의 남의 파일(표식 없음)이 있어 만들지 않는다(Ubuntu).
    Conflict,
}

pub fn shell_menu_kind() -> ShellMenuKind {
    if cfg!(windows) {
        ShellMenuKind::Explorer
    } else if cfg!(target_os = "linux") {
        ShellMenuKind::NautilusScripts
    } else {
        ShellMenuKind::Unsupported
    }
}

/// 메뉴가 실행할 앱 경로. AppImage로 돌면 마운트 경로(실행마다 바뀜) 대신 AppImage 파일(`APPIMAGE`)을 쓴다.
/// `APPIMAGE`가 절대 경로의 있는 파일이 아니면 무시한다.
pub fn launcher_path() -> Option<PathBuf> {
    launcher_from(std::env::var_os("APPIMAGE").map(PathBuf::from))
}

fn launcher_from(appimage: Option<PathBuf>) -> Option<PathBuf> {
    if let Some(path) = appimage
        && path.is_absolute()
        && path.is_file()
        && !path.to_string_lossy().contains(['\0', '\n', '\r', '"'])
    {
        return Some(path);
    }
    crate::current_exe_path()
}

/// 항목 이름과 실행 파일 경로를 등록에 써도 되는지.
fn check(launcher: &Path, label: &str) -> io::Result<()> {
    let bad_label = label.is_empty()
        || label == "."
        || label == ".."
        || label.contains(['/', '\\'])
        || label.chars().any(char::is_control);
    let text = launcher.to_string_lossy();
    let bad_launcher = !launcher.is_absolute() || text.contains(['\0', '\n', '\r', '"']);
    if bad_label || bad_launcher {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "shell menu label or launcher",
        ));
    }
    Ok(())
}

pub fn shell_menu_status(launcher: &Path, label: &str) -> io::Result<ShellMenuState> {
    imp::status_at(&imp::default_root()?, launcher, label)
}

/// 등록한다. 예전 이름의 hiGit 항목은 지운다. 같은 이름의 남의 파일이 있으면 `AlreadyExists`.
pub fn shell_menu_install(launcher: &Path, label: &str) -> io::Result<()> {
    check(launcher, label)?;
    imp::install_at(&imp::default_root()?, launcher, label)
}

/// hiGit이 만든 항목만 지운다. 없으면 아무것도 하지 않는다.
pub fn shell_menu_uninstall() -> io::Result<()> {
    imp::uninstall_at(&imp::default_root()?)
}

#[cfg(windows)]
mod imp {
    //! `HKCU\Software\Classes\Directory\shell\higit`와 `Directory\Background\shell\higit`.

    use std::io;
    use std::path::Path;

    use winreg::RegKey;
    use winreg::enums::HKEY_CURRENT_USER;

    use super::ShellMenuState;

    /// HKCU 아래 경로. 시험은 다른 경로를 루트로 쓴다.
    pub(super) type Root = String;

    const KEYS: [&str; 2] = [
        r"Directory\shell\higit",
        r"Directory\Background\shell\higit",
    ];

    pub(super) fn default_root() -> io::Result<Root> {
        Ok(r"Software\Classes".to_string())
    }

    fn command(launcher: &Path) -> String {
        format!("\"{}\" -- \"%V\"", launcher.display())
    }

    fn icon(launcher: &Path) -> String {
        format!("\"{}\",0", launcher.display())
    }

    pub(super) fn status_at(
        root: &Root,
        launcher: &Path,
        label: &str,
    ) -> io::Result<ShellMenuState> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let mut found = 0;
        let mut matches = 0;
        for key in KEYS {
            let Ok(entry) = hkcu.open_subkey(format!(r"{root}\{key}")) else {
                continue;
            };
            found += 1;
            let verb: String = entry.get_value("MUIVerb").unwrap_or_default();
            let cmd: String = entry
                .open_subkey("command")
                .and_then(|c| c.get_value(""))
                .unwrap_or_default();
            if verb == label && cmd == command(launcher) {
                matches += 1;
            }
        }
        Ok(match (found, matches) {
            (0, _) => ShellMenuState::NotInstalled,
            (2, 2) => ShellMenuState::Installed,
            _ => ShellMenuState::Outdated,
        })
    }

    pub(super) fn install_at(root: &Root, launcher: &Path, label: &str) -> io::Result<()> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        for key in KEYS {
            let (entry, _) = hkcu.create_subkey(format!(r"{root}\{key}"))?;
            entry.set_value("MUIVerb", &label)?;
            entry.set_value("Icon", &icon(launcher))?;
            let (cmd, _) = entry.create_subkey("command")?;
            cmd.set_value("", &command(launcher))?;
        }
        Ok(())
    }

    pub(super) fn uninstall_at(root: &Root) -> io::Result<()> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        for key in KEYS {
            match hkcu.delete_subkey_all(format!(r"{root}\{key}")) {
                Err(e) if e.kind() != io::ErrorKind::NotFound => return Err(e),
                _ => {}
            }
        }
        Ok(())
    }
}

#[cfg(unix)]
mod imp {
    //! `~/.local/share/nautilus/scripts/<항목 이름>` 실행 스크립트. 첫 줄 뒤 표식으로 hiGit 것을 알아본다.

    use std::io::{self, Read, Write};
    use std::os::unix::fs::PermissionsExt;
    use std::path::{Path, PathBuf};

    use super::ShellMenuState;

    /// 스크립트 폴더. 시험은 임시 폴더를 쓴다.
    pub(super) type Root = PathBuf;

    pub(super) const MARKER: &str = "# higit-shell-menu v1";
    /// 표식과 실행 파일 줄을 찾을 때 읽는 앞부분.
    const HEAD_LIMIT: u64 = 4096;

    pub(super) fn default_root() -> io::Result<Root> {
        dirs::data_dir()
            .map(|d| d.join("nautilus").join("scripts"))
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "data dir"))
    }

    /// 작은따옴표 안에 넣을 글자. `'`는 `'\''`로 닫고 열어 넣는다.
    fn quote(text: &str) -> String {
        format!("'{}'", text.replace('\'', r"'\''"))
    }

    fn app_line(launcher: &Path) -> String {
        format!("app={}", quote(&launcher.to_string_lossy()))
    }

    pub(super) fn script(launcher: &Path) -> String {
        format!(
            "#!/bin/sh\n{MARKER}\n\
             # hiGit이 만든 파일입니다. 설정의 \"폴더 오른쪽 클릭 메뉴\"에서 끌 수 있습니다.\n\
             {}\n\
             if [ ! -x \"$app\" ]; then\n  rm -f -- \"$0\"\n  exit 0\nfi\n\
             if [ -n \"$1\" ]; then\n  exec \"$app\" -- \"$1\"\nfi\n\
             exec \"$app\" -- \"$(pwd)\"\n",
            app_line(launcher)
        )
    }

    /// 링크가 아닌 보통 파일이면 앞부분.
    fn head(path: &Path) -> Option<String> {
        let meta = std::fs::symlink_metadata(path).ok()?;
        if !meta.is_file() {
            return None;
        }
        let mut text = String::new();
        std::fs::File::open(path)
            .ok()?
            .take(HEAD_LIMIT)
            .read_to_string(&mut text)
            .ok()?;
        Some(text)
    }

    fn is_ours(text: &str) -> bool {
        text.lines().nth(1) == Some(MARKER)
    }

    /// 폴더 안의 hiGit 스크립트들.
    fn ours(root: &Path) -> Vec<(PathBuf, String)> {
        let Ok(read) = std::fs::read_dir(root) else {
            return Vec::new();
        };
        read.filter_map(Result::ok)
            .filter_map(|e| {
                let path = e.path();
                let text = head(&path)?;
                is_ours(&text).then_some((path, text))
            })
            .collect()
    }

    pub(super) fn status_at(
        root: &Root,
        launcher: &Path,
        label: &str,
    ) -> io::Result<ShellMenuState> {
        let target = root.join(label);
        if std::fs::symlink_metadata(&target).is_ok() && !head(&target).is_some_and(|t| is_ours(&t))
        {
            return Ok(ShellMenuState::Conflict);
        }
        let found = ours(root);
        if found.is_empty() {
            return Ok(ShellMenuState::NotInstalled);
        }
        let exact = found.len() == 1
            && found[0].0 == target
            && found[0].1.lines().any(|l| l == app_line(launcher));
        let mode_ok = std::fs::metadata(&target).is_ok_and(|m| m.permissions().mode() & 0o111 != 0);
        Ok(if exact && mode_ok {
            ShellMenuState::Installed
        } else {
            ShellMenuState::Outdated
        })
    }

    pub(super) fn install_at(root: &Root, launcher: &Path, label: &str) -> io::Result<()> {
        let target = root.join(label);
        if std::fs::symlink_metadata(&target).is_ok() && !head(&target).is_some_and(|t| is_ours(&t))
        {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "shell menu script",
            ));
        }
        std::fs::create_dir_all(root)?;
        // 다른 이름(예전 언어)의 hiGit 스크립트는 지운다.
        for (path, _) in ours(root) {
            if path != target {
                std::fs::remove_file(&path)?;
            }
        }
        let tmp = root.join(format!(".higit-shell-menu-{}.tmp", std::process::id()));
        let result = (|| {
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&tmp)?;
            file.write_all(script(launcher).as_bytes())?;
            file.set_permissions(std::fs::Permissions::from_mode(0o755))?;
            file.sync_all()?;
            std::fs::rename(&tmp, &target)
        })();
        if result.is_err() {
            let _ = std::fs::remove_file(&tmp);
        }
        result
    }

    pub(super) fn uninstall_at(root: &Root) -> io::Result<()> {
        for (path, _) in ours(root) {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_and_launchers_are_checked() {
        let exe = std::env::temp_dir().join("higit");
        assert!(check(&exe, "hiGit으로 열기").is_ok());
        for bad in ["", ".", "..", "a/b", "a\\b", "줄\n바꿈", "탭\t"] {
            assert!(check(&exe, bad).is_err(), "{bad:?}");
        }
        assert!(check(Path::new("relative/higit"), "열기").is_err());
        assert!(check(&std::env::temp_dir().join("a\"b"), "열기").is_err());
    }

    #[test]
    fn launcher_ignores_bad_appimage() {
        let current = crate::current_exe_path();
        assert_eq!(
            launcher_from(Some(PathBuf::from("relative.AppImage"))),
            current
        );
        let missing = std::env::temp_dir().join("없는-higit.AppImage");
        assert_eq!(launcher_from(Some(missing)), current);
        let dir = tempfile::tempdir().unwrap();
        let image = dir.path().join("hiGit.AppImage");
        std::fs::write(&image, b"").unwrap();
        assert_eq!(launcher_from(Some(image.clone())), Some(image));
    }

    #[cfg(unix)]
    mod nautilus {
        use std::os::unix::fs::PermissionsExt;
        use std::path::{Path, PathBuf};

        use super::super::imp::{self, MARKER};
        use super::super::*;

        /// 받은 인자를 한 줄씩 `out`에 쓰는 가짜 앱.
        fn fake_app(dir: &Path, out: &Path) -> PathBuf {
            let app = dir.join("it's 한글 앱");
            std::fs::write(
                &app,
                format!("#!/bin/sh\nprintf '%s\\n' \"$@\" > '{}'\n", out.display()),
            )
            .unwrap();
            std::fs::set_permissions(&app, std::fs::Permissions::from_mode(0o755)).unwrap();
            app
        }

        fn run_script(script: &Path, cwd: &Path, args: &[&str]) {
            let status = std::process::Command::new("sh")
                .arg(script)
                .args(args)
                .current_dir(cwd)
                .status()
                .unwrap();
            assert!(status.success());
        }

        #[test]
        fn install_status_reinstall_uninstall() {
            let tmp = tempfile::tempdir().unwrap();
            let root = tmp.path().join("scripts");
            let out = tmp.path().join("args.txt");
            let app = fake_app(tmp.path(), &out);
            let state = |label: &str| imp::status_at(&root, &app, label).unwrap();
            assert_eq!(state("hiGit으로 열기"), ShellMenuState::NotInstalled);

            imp::install_at(&root, &app, "hiGit으로 열기").unwrap();
            let script = root.join("hiGit으로 열기");
            let text = std::fs::read_to_string(&script).unwrap();
            assert_eq!(text.lines().nth(1), Some(MARKER));
            assert!(text.contains(r"app='"), "{text}");
            assert_eq!(
                std::fs::metadata(&script).unwrap().permissions().mode() & 0o777,
                0o755
            );
            assert_eq!(state("hiGit으로 열기"), ShellMenuState::Installed);
            // 이름(언어)이 바뀌면 고칠 것이 있다. 다시 등록하면 예전 파일은 지운다.
            assert_eq!(state("Open in hiGit"), ShellMenuState::Outdated);
            imp::install_at(&root, &app, "Open in hiGit").unwrap();
            assert!(!script.exists());
            assert_eq!(state("Open in hiGit"), ShellMenuState::Installed);

            // 남의 파일은 그대로 둔다.
            let other = root.join("남의 스크립트");
            std::fs::write(&other, "#!/bin/sh\necho hi\n").unwrap();
            imp::uninstall_at(&root).unwrap();
            assert!(!root.join("Open in hiGit").exists());
            assert!(other.exists());
            assert_eq!(state("Open in hiGit"), ShellMenuState::NotInstalled);
        }

        #[test]
        fn script_passes_dashes_and_the_folder() {
            let tmp = tempfile::tempdir().unwrap();
            let root = tmp.path().join("scripts");
            let out = tmp.path().join("args.txt");
            let app = fake_app(tmp.path(), &out);
            imp::install_at(&root, &app, "열기").unwrap();
            let script = root.join("열기");
            let here = tmp.path().join("지금 폴더");
            std::fs::create_dir(&here).unwrap();

            run_script(&script, &here, &["-고른 폴더"]);
            assert_eq!(std::fs::read_to_string(&out).unwrap(), "--\n-고른 폴더\n");
            run_script(&script, &here, &[]);
            let expected = format!("--\n{}\n", here.canonicalize().unwrap().display());
            assert_eq!(std::fs::read_to_string(&out).unwrap(), expected);
        }

        #[test]
        fn script_removes_itself_when_the_app_is_gone() {
            let tmp = tempfile::tempdir().unwrap();
            let root = tmp.path().join("scripts");
            let out = tmp.path().join("args.txt");
            let app = fake_app(tmp.path(), &out);
            imp::install_at(&root, &app, "열기").unwrap();
            std::fs::remove_file(&app).unwrap();
            run_script(&root.join("열기"), tmp.path(), &[]);
            assert!(!root.join("열기").exists());
            assert!(!out.exists());
        }

        #[test]
        fn foreign_files_and_links_are_not_touched() {
            let tmp = tempfile::tempdir().unwrap();
            let root = tmp.path().join("scripts");
            std::fs::create_dir(&root).unwrap();
            let out = tmp.path().join("args.txt");
            let app = fake_app(tmp.path(), &out);
            std::fs::write(root.join("열기"), "#!/bin/sh\n# 남의 것\n").unwrap();
            assert_eq!(
                imp::status_at(&root, &app, "열기").unwrap(),
                ShellMenuState::Conflict
            );
            let err = imp::install_at(&root, &app, "열기").unwrap_err();
            assert_eq!(err.kind(), std::io::ErrorKind::AlreadyExists);
            assert_eq!(
                std::fs::read_to_string(root.join("열기")).unwrap(),
                "#!/bin/sh\n# 남의 것\n"
            );

            // 표식이 든 파일을 가리키는 링크도 따라가지 않는다.
            let real = tmp.path().join("진짜");
            std::fs::write(&real, imp::script(&app)).unwrap();
            std::os::unix::fs::symlink(&real, root.join("링크")).unwrap();
            assert_eq!(
                imp::status_at(&root, &app, "링크").unwrap(),
                ShellMenuState::Conflict
            );
            assert!(imp::install_at(&root, &app, "링크").is_err());
            imp::uninstall_at(&root).unwrap();
            assert!(real.exists());
            assert!(root.join("링크").exists());
        }
    }

    #[cfg(windows)]
    mod explorer {
        use winreg::RegKey;
        use winreg::enums::HKEY_CURRENT_USER;

        use super::super::imp;
        use super::super::*;

        /// `HKCU\Software\hiGit-test-<무작위>`를 루트로 쓰고 끝나면 지운다.
        struct TestRoot(String);

        impl Drop for TestRoot {
            fn drop(&mut self) {
                let _ = RegKey::predef(HKEY_CURRENT_USER).delete_subkey_all(&self.0);
            }
        }

        fn root() -> TestRoot {
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            TestRoot(format!(
                r"Software\hiGit-test-{}-{nanos}",
                std::process::id()
            ))
        }

        #[test]
        fn install_status_reinstall_uninstall() {
            let root = root();
            let tmp = tempfile::tempdir().unwrap();
            let app = tmp.path().join("한글 폴더").join("higit.exe");
            let moved = tmp.path().join("옮김").join("higit.exe");
            let state =
                |exe: &std::path::Path, label: &str| imp::status_at(&root.0, exe, label).unwrap();
            assert_eq!(state(&app, "hiGit으로 열기"), ShellMenuState::NotInstalled);

            imp::install_at(&root.0, &app, "hiGit으로 열기").unwrap();
            assert_eq!(state(&app, "hiGit으로 열기"), ShellMenuState::Installed);
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            let cmd: String = hkcu
                .open_subkey(format!(r"{}\Directory\shell\higit\command", root.0))
                .unwrap()
                .get_value("")
                .unwrap();
            assert_eq!(cmd, format!("\"{}\" -- \"%V\"", app.display()));
            let icon: String = hkcu
                .open_subkey(format!(r"{}\Directory\Background\shell\higit", root.0))
                .unwrap()
                .get_value("Icon")
                .unwrap();
            assert_eq!(icon, format!("\"{}\",0", app.display()));

            // 실행 파일을 옮기면 고칠 것이 있고, 다시 등록하면 고쳐진다.
            assert_eq!(state(&moved, "hiGit으로 열기"), ShellMenuState::Outdated);
            imp::install_at(&root.0, &moved, "hiGit으로 열기").unwrap();
            assert_eq!(state(&moved, "hiGit으로 열기"), ShellMenuState::Installed);

            // 남의 키는 그대로 둔다.
            let (other, _) = hkcu
                .create_subkey(format!(r"{}\Directory\shell\other", root.0))
                .unwrap();
            other.set_value("MUIVerb", &"남의 것").unwrap();
            imp::uninstall_at(&root.0).unwrap();
            assert_eq!(
                state(&moved, "hiGit으로 열기"),
                ShellMenuState::NotInstalled
            );
            assert!(
                hkcu.open_subkey(format!(r"{}\Directory\shell\other", root.0))
                    .is_ok()
            );
            // 없을 때 지워도 오류가 아니다.
            imp::uninstall_at(&root.0).unwrap();
        }
    }
}
