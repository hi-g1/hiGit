use std::io;
use std::path::PathBuf;

/// 테스트·개발에서 앱 데이터 폴더 위치를 바꾸는 환경 변수.
pub const DATA_DIR_ENV: &str = "HIGIT_DATA_DIR";

/// 앱 데이터 폴더(최근 프로젝트 목록 등, D-032). 없으면 만든다.
///
/// Windows: `%LOCALAPPDATA%\<app_id>`(로밍 아님). Ubuntu: `$XDG_DATA_HOME/<app_id>`,
/// 없거나 상대 경로면 `~/.local/share/<app_id>`. `HIGIT_DATA_DIR`이 있으면 그 폴더를 그대로 쓴다.
pub fn app_data_dir(app_id: &str) -> io::Result<PathBuf> {
    let dir = match std::env::var_os(DATA_DIR_ENV) {
        Some(dir) if !dir.is_empty() => PathBuf::from(dir),
        _ => dirs::data_local_dir()
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, "앱 데이터 폴더를 찾지 못했어요")
            })?
            .join(app_id),
    };
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    // 환경 변수는 프로세스 전체에 걸리므로 한 테스트 안에서 차례로 확인한다.
    #[test]
    fn data_dir_default_and_override() {
        let default = {
            // SAFETY: 이 크레이트에서 HIGIT_DATA_DIR을 읽거나 쓰는 테스트는 이것 하나뿐이다.
            unsafe { std::env::remove_var(DATA_DIR_ENV) };
            app_data_dir("dev.higit.test-only").unwrap()
        };
        assert!(default.is_absolute(), "{default:?}");
        assert!(default.ends_with("dev.higit.test-only"));
        assert!(default.is_dir());
        let _ = std::fs::remove_dir(&default);

        let tmp = tempfile::tempdir().unwrap();
        let wanted = tmp.path().join("데이터 폴더");
        unsafe { std::env::set_var(DATA_DIR_ENV, &wanted) };
        let got = app_data_dir("dev.higit.desktop").unwrap();
        unsafe { std::env::remove_var(DATA_DIR_ENV) };
        assert_eq!(got, wanted);
        assert!(got.is_dir());
    }
}
