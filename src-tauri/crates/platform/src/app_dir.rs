use std::ffi::OsString;
use std::io;
use std::path::PathBuf;

/// 테스트·개발에서 앱 데이터 폴더 위치를 바꾸는 환경 변수.
pub const DATA_DIR_ENV: &str = "HIGIT_DATA_DIR";

/// 앱 데이터 폴더(최근 프로젝트 목록 등, D-032). 없으면 만든다.
///
/// Windows: `%LOCALAPPDATA%\<app_id>`(로밍 아님). Ubuntu: `$XDG_DATA_HOME/<app_id>`,
/// 없거나 상대 경로면 `~/.local/share/<app_id>`. `HIGIT_DATA_DIR`이 있으면 그 폴더를 그대로 쓴다.
pub fn app_data_dir(app_id: &str) -> io::Result<PathBuf> {
    let dir = resolve_data_dir(
        std::env::var_os(DATA_DIR_ENV),
        dirs::data_local_dir(),
        app_id,
    )?;
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// 다른 사용자가 읽지 못하게 파일을 쓴다(journal 서명 키 등, D-063). 임시 파일에 쓰고 바꿔 넣는다.
/// Linux: 권한 0600. Windows: 사용자 앱 데이터 폴더의 기본 권한(본인만)을 따른다.
pub fn write_private_file(path: &std::path::Path, bytes: &[u8]) -> io::Result<()> {
    use std::io::Write;
    let tmp = path.with_extension("tmp");
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&tmp)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    drop(file);
    std::fs::rename(&tmp, path)
}

/// 다른 사용자가 읽지 못하게 덧붙여 쓸 파일을 연다(로그 파일, D-078). 없으면 만든다.
/// Linux: 새로 만들 때 권한 0600. Windows: 사용자 앱 데이터 폴더의 기본 권한을 따른다.
pub fn open_private_append(path: &std::path::Path) -> io::Result<std::fs::File> {
    let mut options = std::fs::OpenOptions::new();
    options.append(true).create(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path)
}

/// 사용자 홈 폴더. 오류 원문에서 개인 경로를 가릴 때 쓴다(D-040).
pub fn home_dir() -> Option<PathBuf> {
    dirs::home_dir()
}

/// 환경을 읽지 않는 위치 계산. 테스트가 환경 변수를 바꾸지 않고 부를 수 있게 나눴다.
fn resolve_data_dir(
    override_dir: Option<OsString>,
    local_data: Option<PathBuf>,
    app_id: &str,
) -> io::Result<PathBuf> {
    match override_dir {
        Some(dir) if !dir.is_empty() => Ok(PathBuf::from(dir)),
        _ => local_data.map(|base| base.join(app_id)).ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "앱 데이터 폴더를 찾지 못했어요")
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn private_append_is_owner_only() {
        use std::io::Write;
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.log");
        open_private_append(&path)
            .unwrap()
            .write_all(b"1\n")
            .unwrap();
        open_private_append(&path)
            .unwrap()
            .write_all(b"2\n")
            .unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "1\n2\n");
        let mode = std::fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    #[test]
    fn default_location_joins_app_id() {
        let base = std::env::temp_dir().join("local");
        let got = resolve_data_dir(None, Some(base.clone()), "dev.higit.desktop").unwrap();
        assert_eq!(got, base.join("dev.higit.desktop"));
        // 빈 값은 덮어쓰기로 보지 않는다.
        let got = resolve_data_dir(Some(OsString::new()), Some(base.clone()), "a").unwrap();
        assert_eq!(got, base.join("a"));
    }

    #[test]
    fn override_wins_and_missing_base_is_an_error() {
        let wanted = std::env::temp_dir().join("데이터 폴더");
        let got = resolve_data_dir(Some(wanted.clone().into()), None, "a").unwrap();
        assert_eq!(got, wanted);
        let err = resolve_data_dir(None, None, "a").unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn private_file_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("비밀.key");
        write_private_file(&path, b"abc").unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"abc");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&path).unwrap().permissions().mode();
            assert_eq!(mode & 0o777, 0o600);
        }
    }

    #[test]
    fn real_default_is_absolute() {
        // 폴더를 만들지 않고 위치만 확인한다.
        if let Some(base) = dirs::data_local_dir() {
            let got = resolve_data_dir(None, Some(base), "dev.higit.test-only").unwrap();
            assert!(got.is_absolute(), "{got:?}");
        }
    }
}
