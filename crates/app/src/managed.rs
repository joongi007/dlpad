//! Native managed-tool commands; all filesystem/network work runs in tasks.
use crate::catalog as c;
use dlpad_engine::tools::{
    InstallManifest, InstalledTool, Ownership, StagingRecovery, ToolManager,
};
use iced::widget::{button, column, row, scrollable, text};
use iced::{Element, Length, Task};
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

/// Product tool identity; selections must update only the corresponding field.
#[allow(dead_code)] // Shared module: the Iced entry still uses its yt-dlp-only panel.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum ToolId {
    #[default]
    YtDlp,
    Deno,
    Ffmpeg,
}
#[allow(dead_code)] // Runtime selection is consumed by the CPU/accessible entrypoints.
impl ToolId {
    pub(crate) fn id(self) -> &'static str {
        match self {
            Self::YtDlp => "yt-dlp",
            Self::Deno => "deno",
            Self::Ffmpeg => "ffmpeg",
        }
    }
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::YtDlp => c::TOOL_YT_DLP,
            Self::Deno => c::TOOL_DENO,
            Self::Ffmpeg => c::TOOL_FFMPEG,
        }
    }
    pub(crate) fn install_available(self) -> bool {
        match self {
            Self::YtDlp => candidate().is_ok(),
            Self::Deno | Self::Ffmpeg => {
                crate::managed_candidates::archive_candidate(self).is_some()
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Refresh,
    Install,
    Select(InstallManifest),
    Remove(InstallManifest),
    Cancel,
    UseActive,
    UseVerified(Result<PathBuf, String>),
    Finished(Result<Inventory, String>),
}
#[derive(Debug, Clone)]
pub struct Inventory {
    installed: Vec<InstalledTool>,
    active: Option<InstalledTool>,
    recovery: StagingRecovery,
}
#[derive(Default)]
pub struct Panel {
    pub busy: bool,
    inventory: Option<Inventory>,
    error: Option<String>,
    cancel: Arc<AtomicBool>,
}
enum Action {
    Refresh,
    Install,
    Select(InstallManifest),
    Remove(InstallManifest),
}

pub(crate) fn tool_error(error: dlpad_engine::tools::ToolError) -> String {
    use dlpad_engine::tools::ToolError::*;
    match error {
        Http => c::TOOL_HTTP_FAILED,
        HashMismatch | UnexpectedSize | DownloadLimit => c::TOOL_HASH_FAILED,
        VersionMismatch => c::TOOL_VERSION_FAILED,
        Cancelled => c::TOOL_CANCELLED,
        TimedOut => c::TOOL_TIMEOUT,
        ActiveVersion => c::TOOL_ACTIVE_REMOVE,
        ExecutionPending => c::TOOL_EXECUTION_PENDING,
        InventoryLimit => c::TOOL_INVENTORY_LIMIT,
        _ => c::TOOL_FAILED,
    }
    .into()
}
pub(crate) fn candidate() -> Result<(InstallManifest, u64), String> {
    let (target, asset, executable, bytes, sha256) =
        if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
            (
                "x86_64-unknown-linux-gnu",
                "yt-dlp_linux",
                "yt-dlp",
                40_446_224,
                "58162f9bfdc27458ea47bfcb311cf47028f17d8154a8bf7d689861d46399230a",
            )
        } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
            (
                "x86_64-pc-windows-msvc",
                "yt-dlp.exe",
                "yt-dlp.exe",
                17_840_399,
                "66674953fe251b89f4d08c5f0e35e0728679bd67ab3d7d05c0562af101dd3e7a",
            )
        } else if cfg!(all(
            target_os = "macos",
            any(target_arch = "x86_64", target_arch = "aarch64")
        )) {
            (
                "universal-apple-darwin",
                "yt-dlp_macos",
                "yt-dlp",
                37_146_048,
                "0f192b7ec147ab6288885d6351d9ab67367640029b4377576ef46dd79cf7b202",
            )
        } else {
            return Err(c::TOOL_UNSUPPORTED.into());
        };
    Ok((
        InstallManifest {
            tool: "yt-dlp".into(),
            version: "2026.08.19".into(),
            target: target.into(),
            source_url: format!(
                "https://github.com/yt-dlp/yt-dlp/releases/download/2026.08.19/{asset}"
            ),
            sha256: sha256.into(),
            license: "GPL-3.0-or-later; THIRD_PARTY_LICENSES.txt at tag 2026.08.19".into(),
            ownership: Ownership::Managed,
            executable: executable.into(),
        },
        bytes,
    ))
}
fn perform(data: PathBuf, action: Action, cancel: Arc<AtomicBool>) -> Result<Inventory, String> {
    let mut manager = ToolManager::new(&data.join("managed-tools")).map_err(tool_error)?;
    let recovery = manager.recover_staging().map_err(tool_error)?;
    match action {
        Action::Refresh => (),
        Action::Install => {
            let (manifest, bytes) = candidate()?;
            manager
                .download_and_install(manifest, bytes, &cancel)
                .map_err(tool_error)?;
        }
        Action::Select(manifest) => {
            manager.select_existing(&manifest).map_err(tool_error)?;
        }
        Action::Remove(manifest) => {
            manager.remove(&manifest).map_err(tool_error)?;
        }
    }
    Ok(Inventory {
        installed: manager.installed("yt-dlp").map_err(tool_error)?,
        active: manager.active_installed("yt-dlp").map_err(tool_error)?,
        recovery,
    })
}
impl Panel {
    pub fn update(
        &mut self,
        message: Message,
        running: bool,
        data: Option<PathBuf>,
    ) -> Task<Message> {
        if let Message::UseVerified(result) = message {
            self.busy = false;
            self.error = result.err();
            return Task::none();
        }
        if let Message::Finished(result) = message {
            self.busy = false;
            match result {
                Ok(inventory) => {
                    self.inventory = Some(inventory);
                    self.error = None;
                }
                Err(error) => self.error = Some(error),
            }
            return Task::none();
        }
        if matches!(message, Message::Cancel) {
            if self.busy {
                self.cancel.store(true, Ordering::Release);
            }
            return Task::none();
        }
        if running || self.busy {
            return Task::none();
        }
        if matches!(message, Message::UseActive) {
            let Some(data) = data else {
                self.error = Some(c::DATA_REQUIRED.into());
                return Task::none();
            };
            self.busy = true;
            self.error = None;
            return Task::perform(
                async move {
                    ToolManager::new(&data.join("managed-tools"))
                        .and_then(|manager| manager.active_installed("yt-dlp"))
                        .map_err(tool_error)?
                        .map(|tool| tool.executable)
                        .ok_or_else(|| c::TOOL_EMPTY.to_owned())
                },
                Message::UseVerified,
            );
        }
        let action = match message {
            Message::Refresh => Action::Refresh,
            Message::Install => Action::Install,
            Message::Select(manifest) => Action::Select(manifest),
            Message::Remove(manifest) => Action::Remove(manifest),
            _ => return Task::none(),
        };
        let Some(data) = data else {
            self.error = Some(c::DATA_REQUIRED.into());
            return Task::none();
        };
        self.busy = true;
        self.error = None;
        self.cancel = Arc::new(AtomicBool::new(false));
        let cancel = self.cancel.clone();
        Task::perform(
            async move { perform(data, action, cancel) },
            Message::Finished,
        )
    }
    pub fn view(&self, running: bool) -> Element<'_, Message> {
        let enabled = !self.busy && !running;
        let mut list = column![
            text(c::TOOLS).size(20),
            text(c::TOOL_HELP),
            text(c::TOOL_LICENSE),
            row![
                button(c::TOOL_REFRESH).on_press_maybe(enabled.then_some(Message::Refresh)),
                button(c::TOOL_INSTALL)
                    .on_press_maybe((enabled && candidate().is_ok()).then_some(Message::Install)),
                button(c::TOOL_CANCEL).on_press_maybe(self.busy.then_some(Message::Cancel))
            ]
            .spacing(8)
        ]
        .spacing(12);
        if let Ok((manifest, bytes)) = candidate() {
            list = list.push(text(format!(
                "yt-dlp {} · {:.1} MiB · {}",
                manifest.version,
                bytes as f64 / 1048576.0,
                manifest.target
            )));
        } else {
            list = list.push(text(c::TOOL_UNSUPPORTED));
        }
        if self.busy {
            list = list.push(text(c::TOOL_BUSY));
        }
        if running {
            list = list.push(text(c::TOOL_RUNNING));
        }
        if let Some(error) = &self.error {
            list = list.push(text(error));
        }
        if let Some(inventory) = &self.inventory {
            if inventory.recovery.removed > 0 {
                list = list.push(text(format!(
                    "{} {}",
                    c::TOOL_STAGING_RECOVERED,
                    inventory.recovery.removed
                )));
            }
            if inventory.recovery.preserved > 0 {
                list = list.push(text(format!(
                    "{} {}",
                    c::TOOL_STAGING_PRESERVED,
                    inventory.recovery.preserved
                )));
            }
            if inventory.installed.is_empty() {
                list = list.push(text(c::TOOL_EMPTY));
            }
            for tool in &inventory.installed {
                let active = inventory.active.as_ref().is_some_and(|selected| {
                    selected.manifest.version == tool.manifest.version
                        && selected.manifest.target == tool.manifest.target
                });
                list = list.push(
                    row![
                        text(format!(
                            "{} · {:.1} MiB · {}",
                            tool.manifest.version,
                            tool.bytes as f64 / 1048576.0,
                            if active {
                                c::TOOL_ACTIVE
                            } else {
                                c::TOOL_INACTIVE
                            }
                        ))
                        .width(Length::Fill),
                        button(c::TOOL_SELECT).on_press_maybe(
                            (enabled && !active).then(|| Message::Select(tool.manifest.clone()))
                        ),
                        button(c::TOOL_REMOVE).on_press_maybe(
                            (enabled && !active).then(|| Message::Remove(tool.manifest.clone()))
                        )
                    ]
                    .spacing(8),
                );
            }
            if let Some(active) = &inventory.active {
                list = list
                    .push(text(active.executable.display().to_string()))
                    .push(
                        button(c::TOOL_USE).on_press_maybe(enabled.then_some(Message::UseActive)),
                    );
            }
        }
        scrollable(list).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn install_is_blocked_during_download_and_cancel_only_targets_owned_request() {
        let mut panel = Panel::default();
        let initial = panel.cancel.clone();
        let _ = panel.update(
            Message::Install,
            true,
            Some(PathBuf::from("unused-test-data")),
        );
        assert!(!panel.busy);
        assert!(Arc::ptr_eq(&initial, &panel.cancel));
        let _ = panel.update(Message::Cancel, false, None);
        assert!(!panel.cancel.load(Ordering::Acquire));
        panel.busy = true;
        let _ = panel.update(Message::Cancel, false, None);
        assert!(panel.cancel.load(Ordering::Acquire));
        let _ = panel.update(
            Message::Install,
            false,
            Some(PathBuf::from("unused-test-data")),
        );
        assert!(Arc::ptr_eq(&initial, &panel.cancel));
    }
    #[test]
    fn failed_refresh_keeps_last_inventory_and_verified_path_failure_releases_gate() {
        let mut panel = Panel::default();
        let _ = panel.update(
            Message::Finished(Ok(Inventory {
                installed: vec![],
                active: None,
                recovery: StagingRecovery::default(),
            })),
            false,
            None,
        );
        panel.busy = true;
        let _ = panel.update(Message::Finished(Err(c::TOOL_FAILED.into())), false, None);
        assert!(!panel.busy);
        assert!(panel.inventory.is_some());
        assert!(panel.error.is_some());
        panel.busy = true;
        let _ = panel.update(
            Message::UseVerified(Err(c::TOOL_FAILED.into())),
            false,
            None,
        );
        assert!(!panel.busy);
        assert!(panel.error.is_some());
    }
}
