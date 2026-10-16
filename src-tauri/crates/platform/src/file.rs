//! 링크를 따라가지 않는 파일 열기(D-037). OS마다 플래그가 달라 여기 둔다.

use std::fs::{File, OpenOptions};
use std::io;
use std::path::Path;

/// 읽기용으로 일반 파일만 연다. 마지막 이름이 심볼릭 링크(Windows는 재분석 지점)거나 일반 파일이 아니면
/// 실패한다. "링크가 아님"을 본 뒤 여는 사이에 링크로 바뀌는 틈을 막는다.
pub fn open_regular_no_follow(path: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use rustix::fs::OFlags;
        use std::os::unix::fs::OpenOptionsExt;
        // FIFO를 열 때 멈추지 않게 NONBLOCK도 준다. 아래에서 일반 파일이 아니면 버린다.
        let flags = (OFlags::NOFOLLOW | OFlags::NONBLOCK).bits();
        options.custom_flags(i32::try_from(flags).map_err(io::Error::other)?);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        /// 재분석 지점(링크·정션)을 따라가지 않고 그 자체를 연다.
        const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
        options.custom_flags(FILE_FLAG_OPEN_REPARSE_POINT);
    }
    let file = options.open(path)?;
    let meta = file.metadata()?;
    if meta.file_type().is_symlink() || !meta.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "일반 파일이 아니에요",
        ));
    }
    Ok(file)
}

/// 실행 권한 비트가 있는지(D-216). Windows는 비트가 없어 늘 거짓이다(확장자로 가른다).
pub fn is_executable(meta: &std::fs::Metadata) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        meta.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        let _ = meta;
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn opens_a_regular_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("한글 파일.txt");
        std::fs::write(&path, "내용").unwrap();
        let mut text = String::new();
        open_regular_no_follow(&path)
            .unwrap()
            .read_to_string(&mut text)
            .unwrap();
        assert_eq!(text, "내용");
    }

    #[test]
    fn refuses_links_and_folders() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("원본.txt");
        std::fs::write(&target, "비밀").unwrap();
        let link = dir.path().join("링크.txt");
        match crate::symlink_file(&target, &link) {
            Ok(()) => assert!(open_regular_no_follow(&link).is_err()),
            Err(e) => eprintln!("이 환경에서는 심볼릭 링크를 만들 수 없습니다: {e}"),
        }
        assert!(open_regular_no_follow(dir.path()).is_err());
        assert!(open_regular_no_follow(&dir.path().join("없음")).is_err());
    }
}
