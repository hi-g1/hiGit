//! 폴더 오른쪽 클릭 메뉴 켬·끔과 처음 등록(D-214). 실제 등록은 `platform::shell_menu`가 하고, 여기서는 이름 검사와
//! "사람이 정했는지"(설정 `shell_menu`)를 다룬다. 시험은 [`ShellMenuPort`] 가짜 구현을 쓴다.

use std::io;
use std::path::{Path, PathBuf};

use crate::view::{ShellMenuKindView, ShellMenuView};
use crate::{CoreError, Result};

/// 항목 이름 상한(바이트).
pub const LABEL_MAX_BYTES: usize = 64;

/// OS 메뉴 등록. 기본은 [`PlatformShellMenu`].
pub trait ShellMenuPort: Send + Sync {
    fn kind(&self) -> ShellMenuKindView;
    /// 메뉴가 실행할 앱 경로. 모르면 `None`(등록하지 않는다).
    fn launcher(&self) -> Option<PathBuf>;
    fn status(&self, launcher: &Path, label: &str) -> io::Result<platform::ShellMenuState>;
    fn install(&self, launcher: &Path, label: &str) -> io::Result<()>;
    fn uninstall(&self) -> io::Result<()>;
}

pub struct PlatformShellMenu;

impl ShellMenuPort for PlatformShellMenu {
    fn kind(&self) -> ShellMenuKindView {
        match platform::shell_menu_kind() {
            platform::ShellMenuKind::Explorer => ShellMenuKindView::Explorer,
            platform::ShellMenuKind::NautilusScripts => ShellMenuKindView::NautilusScripts,
            platform::ShellMenuKind::Unsupported => ShellMenuKindView::Unsupported,
        }
    }

    fn launcher(&self) -> Option<PathBuf> {
        platform::launcher_path()
    }

    fn status(&self, launcher: &Path, label: &str) -> io::Result<platform::ShellMenuState> {
        platform::shell_menu_status(launcher, label)
    }

    fn install(&self, launcher: &Path, label: &str) -> io::Result<()> {
        platform::shell_menu_install(launcher, label)
    }

    fn uninstall(&self) -> io::Result<()> {
        platform::shell_menu_uninstall()
    }
}

/// 화면 언어의 항목 이름. 비어 있지 않고 64바이트 이하, 제어 문자·경로 구분자 없음.
pub(crate) fn check_label(label: &str) -> Result<()> {
    let bad = label.trim().is_empty()
        || label.len() > LABEL_MAX_BYTES
        || label.contains(['/', '\\'])
        || label.chars().any(char::is_control);
    if bad {
        return Err(CoreError::InvalidInput("shell menu label"));
    }
    Ok(())
}

fn failed(e: &io::Error) -> CoreError {
    CoreError::ShellMenu(format!("{:?}", e.kind()))
}

/// 지금 등록 상태. 실행 파일·이름이 달라도(고칠 것이 있어도) 켜진 것으로 본다.
pub(crate) fn view(port: &dyn ShellMenuPort, label: &str) -> ShellMenuView {
    let kind = port.kind();
    let installed = kind != ShellMenuKindView::Unsupported
        && port.launcher().is_some_and(|launcher| {
            matches!(
                port.status(&launcher, label),
                Ok(platform::ShellMenuState::Installed | platform::ShellMenuState::Outdated)
            )
        });
    ShellMenuView { kind, installed }
}

/// 앱을 켤 때. 값이 없고 `auto`(릴리스 빌드)면 등록하고 켬으로 기록, 켬인데 실행 파일·이름이 바뀌었으면 다시
/// 등록, 끔이면 아무것도 하지 않는다. 등록 실패는 다음 실행에 다시 해 본다.
pub(crate) fn sync(
    port: &dyn ShellMenuPort,
    chosen: Option<bool>,
    label: &str,
    auto: bool,
) -> Result<Option<bool>> {
    check_label(label)?;
    if port.kind() == ShellMenuKindView::Unsupported {
        return Ok(chosen);
    }
    let Some(launcher) = port.launcher() else {
        return Ok(chosen);
    };
    match chosen {
        None if auto => {
            port.install(&launcher, label).map_err(|e| failed(&e))?;
            Ok(Some(true))
        }
        Some(true) => {
            if !matches!(
                port.status(&launcher, label),
                Ok(platform::ShellMenuState::Installed)
            ) {
                port.install(&launcher, label).map_err(|e| failed(&e))?;
            }
            Ok(chosen)
        }
        None | Some(false) => Ok(chosen),
    }
}

/// 설정에서 켜고 끈다. 성공하면 새 값을 돌려준다.
pub(crate) fn set(port: &dyn ShellMenuPort, on: bool, label: &str) -> Result<bool> {
    check_label(label)?;
    if on {
        let launcher = port
            .launcher()
            .ok_or_else(|| CoreError::ShellMenu("launcher".into()))?;
        port.install(&launcher, label).map_err(|e| failed(&e))?;
    } else {
        port.uninstall().map_err(|e| failed(&e))?;
    }
    Ok(on)
}
