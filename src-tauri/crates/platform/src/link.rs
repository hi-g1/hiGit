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

/// 폴더 링크를 만든다. Windows는 심볼릭 링크가 안 되면 권한 없이 되는 정션으로 만든다.
/// 테스트 도우미(저장소 밖을 가리키는 폴더 링크 시험).
pub fn symlink_dir(target: &Path, link: &Path) -> io::Result<()> {
    #[cfg(windows)]
    {
        if std::os::windows::fs::symlink_dir(target, link).is_ok() {
            return Ok(());
        }
        let status = std::process::Command::new("cmd")
            .arg("/C")
            .arg("mklink")
            .arg("/J")
            .arg(link)
            .arg(target)
            .output()?;
        if status.status.success() {
            Ok(())
        } else {
            Err(io::Error::other(
                String::from_utf8_lossy(&status.stderr).into_owned(),
            ))
        }
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
