//! 이미지 비교 조회를 실제 git으로 시험한다(D-206).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;
use std::sync::Arc;

use base64::Engine as _;
use higit_core::view::{ErrorCode, ErrorView, ImageMime, ImagePairView, ImageRequest, OpenHow};
use higit_core::{Core, GitBackend};

fn git(dir: &Path, args: &[&str]) -> String {
    let out = std::process::Command::new("git")
        .args(["-c", "core.autocrlf=false", "-c", "commit.gpgsign=false"])
        .args(["-c", "user.name=t", "-c", "user.email=t@example.com"])
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap().trim().to_string()
}

fn png(fill: u8, len: usize) -> Vec<u8> {
    let mut b = b"\x89PNG\r\n\x1a\n".to_vec();
    b.resize(len.max(8), fill);
    b
}

fn repo() -> tempfile::TempDir {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["config", "core.autocrlf", "false"]);
    std::fs::create_dir(dir.join("img")).unwrap();
    std::fs::write(dir.join("img/logo.png"), png(1, 64)).unwrap();
    std::fs::write(dir.join("old.gif"), b"GIF89a\x01\0\x01\0").unwrap();
    std::fs::write(dir.join("readme.txt"), "글").unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "처음"]);
    repo
}

fn core(dir: &Path) -> Core {
    let backend = git::CliBackend::new("git", Arc::new(|_| {}));
    let core = Core::with_backend(Arc::new(backend) as Arc<dyn GitBackend>).unwrap();
    core.open_project(dir, OpenHow::Picked).unwrap();
    core
}

fn pair(
    core: &Core,
    dir: &Path,
    path: &str,
    orig: Option<&str>,
    commit: Option<&str>,
) -> higit_core::Result<ImagePairView> {
    core.queries().image_pair(
        dir,
        &ImageRequest {
            path: path.into(),
            orig_path: orig.map(str::to_string),
            commit: commit.map(str::to_string),
        },
    )
}

fn decode(data: &str) -> Vec<u8> {
    base64::engine::general_purpose::STANDARD
        .decode(data)
        .unwrap()
}

#[test]
fn changed_added_deleted_and_renamed_images() {
    let repo = repo();
    let dir = repo.path();
    let core = core(dir);
    // 고침
    std::fs::write(dir.join("img/logo.png"), png(2, 80)).unwrap();
    let v = pair(&core, dir, "img/logo.png", None, None).unwrap();
    let (before, after) = (v.before.unwrap(), v.after.unwrap());
    assert_eq!((before.mime, after.mime), (ImageMime::Png, ImageMime::Png));
    assert_eq!(decode(&before.data), png(1, 64));
    assert_eq!(decode(&after.data), png(2, 80));
    assert_eq!(after.bytes, 80);
    // 새 파일
    std::fs::write(dir.join("새 그림.webp"), b"RIFF\x10\0\0\0WEBPVP8 data").unwrap();
    let v = pair(&core, dir, "새 그림.webp", None, None).unwrap();
    assert!(v.before.is_none());
    assert_eq!(v.after.unwrap().mime, ImageMime::Webp);
    // 지움
    std::fs::remove_file(dir.join("old.gif")).unwrap();
    let v = pair(&core, dir, "old.gif", None, None).unwrap();
    assert_eq!(v.before.unwrap().mime, ImageMime::Gif);
    assert!(v.after.is_none());
    // 이름 바꿈: 이전 쪽은 원래 경로에서
    git(dir, &["checkout", "--", "old.gif"]);
    git(dir, &["mv", "old.gif", "new.gif"]);
    let v = pair(&core, dir, "new.gif", Some("old.gif"), None).unwrap();
    assert!(v.before.is_some() && v.after.is_some());
}

#[test]
fn inside_a_version_and_on_the_first_version() {
    let repo = repo();
    let dir = repo.path();
    let first = git(dir, &["rev-parse", "HEAD"]);
    std::fs::write(dir.join("img/logo.png"), png(3, 70)).unwrap();
    git(dir, &["commit", "-q", "-am", "그림 고침"]);
    let second = git(dir, &["rev-parse", "HEAD"]);
    let core = core(dir);
    let v = pair(&core, dir, "img/logo.png", None, Some(&second)).unwrap();
    assert_eq!(decode(&v.before.unwrap().data), png(1, 64));
    assert_eq!(decode(&v.after.unwrap().data), png(3, 70));
    // 첫 버전은 이전 쪽이 없다.
    let v = pair(&core, dir, "img/logo.png", None, Some(&first)).unwrap();
    assert!(v.before.is_none() && v.after.is_some());
}

#[test]
fn not_images_size_limit_and_unsafe_paths() {
    let repo = repo();
    let dir = repo.path();
    let core = core(dir);
    // 한쪽이라도 이미지가 아니면 둘 다 없다.
    std::fs::write(dir.join("readme.txt"), "더 많은 글").unwrap();
    let v = pair(&core, dir, "readme.txt", None, None).unwrap();
    assert!(v.before.is_none() && v.after.is_none() && !v.too_large);
    // 3MB까지는 되고 넘으면 too_large.
    std::fs::write(dir.join("img/logo.png"), png(0, 3 * 1024 * 1024)).unwrap();
    let v = pair(&core, dir, "img/logo.png", None, None).unwrap();
    assert!(!v.too_large && v.after.is_some());
    std::fs::write(dir.join("img/logo.png"), png(0, 3 * 1024 * 1024 + 1)).unwrap();
    let v = pair(&core, dir, "img/logo.png", None, None).unwrap();
    assert!(v.too_large && v.before.is_none() && v.after.is_none());
    // 저장소 밖 경로는 받지 않는다.
    for bad in ["../x.png", "/etc/passwd", "img/../../x.png"] {
        let err = pair(&core, dir, bad, None, None).unwrap_err();
        assert_eq!(ErrorView::from(&err).code, ErrorCode::InvalidInput, "{bad}");
    }
    // 상태에 없는 파일(바뀌지 않음)은 지금 쪽을 읽지 않는다.
    std::fs::write(dir.join("img/logo.png"), png(1, 64)).unwrap();
    let v = pair(&core, dir, "img/logo.png", None, None).unwrap();
    assert!(v.after.is_none());
}

#[cfg(unix)]
#[test]
fn links_are_not_followed() {
    let repo = repo();
    let dir = repo.path();
    let outside = tempfile::tempdir().unwrap();
    std::fs::write(outside.path().join("secret.png"), png(9, 32)).unwrap();
    std::os::unix::fs::symlink(outside.path().join("secret.png"), dir.join("link.png")).unwrap();
    // 중간 폴더가 링크인 경우도.
    std::os::unix::fs::symlink(outside.path(), dir.join("linkdir")).unwrap();
    let core = core(dir);
    let v = pair(&core, dir, "link.png", None, None).unwrap();
    assert!(v.after.is_none());
    let v = pair(&core, dir, "linkdir/secret.png", None, None).unwrap();
    assert!(v.after.is_none());
}
