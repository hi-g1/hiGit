use std::io;
use std::path::Path;

/// 파일 심볼릭 링크를 만든다. OS마다 방법이 달라 여기 둔다.
/// Windows는 개발자 모드나 관리자 권한이 없으면 실패할 수 있다(테스트는 실패를 받아들인다).
pub fn symlink_file(target: &Path, link: &Path) -> io::Result<()> {
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_file(target, link)
    }
    #[cfg(not(windows))]
    {
        std::os::unix::fs::symlink(target, link)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_a_link_or_reports_why() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("원본.txt");
        std::fs::write(&target, "내용").unwrap();
        let link = dir.path().join("링크.txt");
        match symlink_file(&target, &link) {
            Ok(()) => assert_eq!(std::fs::read_to_string(&link).unwrap(), "내용"),
            Err(e) => eprintln!("이 환경에서는 심볼릭 링크를 만들 수 없습니다: {e}"),
        }
    }
}
