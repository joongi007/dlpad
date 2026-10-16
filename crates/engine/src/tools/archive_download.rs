//! Worker-only pinned archive transfer; successful installation remains inactive.
use super::{
    direct_download::{
        DownloadTransport, HttpsTransport, OwnedDirectory, checkpoint, source_policy,
        transfer_to_owned_file,
    },
    staging_recovery::file_evidence,
    *,
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};
/// Original read-only notice capability retained through installation.
pub(super) struct VerifiedNoticeFile {
    pub(super) component: String,
    pub(super) path: PathBuf,
    pub(super) file: File,
}
static SEQUENCE: AtomicU64 = AtomicU64::new(0);
impl ToolManager {
    /// Stream a validated pinned HTTPS package and install its inactive component set.
    /// Runtime/version and publisher trust are separate activation prerequisites.
    pub fn download_archive_and_install(
        &mut self,
        package: ArchivePackage,
        cancel: &Arc<AtomicBool>,
    ) -> Result<InstalledPackage, ToolError> {
        self.install_archive_download(
            package,
            cancel,
            &mut HttpsTransport,
            Duration::from_secs(120),
        )
    }
    /// Preserve the per-tool bound without resetting the caller's absolute deadline.
    pub fn download_archive_and_install_until(
        &mut self,
        package: ArchivePackage,
        cancel: &Arc<AtomicBool>,
        deadline: Instant,
    ) -> Result<InstalledPackage, ToolError> {
        let limit = Instant::now()
            .checked_add(Duration::from_secs(120))
            .ok_or(ToolError::TimedOut)?;
        self.install_archive_download_until(
            package,
            cancel,
            &mut HttpsTransport,
            deadline.min(limit),
        )
    }
    pub(super) fn install_archive_download(
        &mut self,
        package: ArchivePackage,
        cancel: &Arc<AtomicBool>,
        transport: &mut dyn DownloadTransport,
        timeout: Duration,
    ) -> Result<InstalledPackage, ToolError> {
        let deadline = Instant::now()
            .checked_add(timeout)
            .ok_or(ToolError::TimedOut)?;
        self.install_archive_download_until(package, cancel, transport, deadline)
    }
    fn install_archive_download_until(
        &mut self,
        package: ArchivePackage,
        cancel: &Arc<AtomicBool>,
        transport: &mut dyn DownloadTransport,
        deadline: Instant,
    ) -> Result<InstalledPackage, ToolError> {
        package.validate()?;
        source_policy(&package.source_url)?;
        checkpoint(cancel, deadline)?;
        let _lock = self.acquire_install_lock()?;
        self.verify_install_inventory_capacity()?;
        // Reserve transfer plus expanded output/metadata before creating the source.
        let reserve = package.components.iter().try_fold(
            package
                .archive_bytes
                .checked_add(2 * MANIFEST_LIMIT)
                .ok_or(ToolError::DownloadLimit)?,
            |sum, c| sum.checked_add(c.bytes).ok_or(ToolError::DownloadLimit),
        )?;
        let reserve = package.notices.iter().try_fold(reserve, |sum, notice| {
            let component = package
                .components
                .iter()
                .find(|c| c.name == notice.component)
                .ok_or(ToolError::InvalidManifest)?;
            sum.checked_add(component.bytes)
                .ok_or(ToolError::DownloadLimit)
        })?;
        self.reserve_staging(reserve)?;
        let mut stage = OwnedDirectory::create(self.root.join(format!(
            ".staging-archive-transfer-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        )))?;
        let archive_name = if matches!(package.layout, Some(ArchiveLayout::TarXz { .. })) {
            "archive.tar.xz"
        } else {
            "archive.zip"
        };
        let path = stage.path.join(archive_name);
        let mut output = stage.file(archive_name)?;
        self.record_package_staging(
            &stage.path,
            &stage.identity,
            archive_name,
            vec![file_evidence(
                archive_name,
                &output,
                0,
                format!("{:x}", Sha256::digest([])),
            )?],
        )?;
        let mut stream = transport.open(&package.source_url, checkpoint(cancel, deadline)?)?;
        transfer_to_owned_file(
            &mut *stream,
            &mut output,
            package.archive_bytes,
            &package.archive_sha256,
            cancel,
            deadline,
            |file, bytes, digest| {
                self.record_package_staging(
                    &stage.path,
                    &stage.identity,
                    archive_name,
                    vec![file_evidence(archive_name, file, bytes, digest)?],
                )
            },
        )?;
        drop(stream);
        drop(output);
        stage.seal_executable(archive_name)?;
        stage.check()?;
        let source = native_files::open_regular(&path)?;
        let mut evidence = vec![file_evidence(
            archive_name,
            &source,
            package.archive_bytes,
            package.archive_sha256.clone(),
        )?];
        let mut notices = Vec::with_capacity(package.notices.len());
        for (index, notice) in package.notices.iter().enumerate() {
            checkpoint(cancel, deadline)?;
            stage.check()?;
            source_policy(&notice.source_url)?;
            let component = package
                .components
                .iter()
                .find(|c| c.name == notice.component)
                .ok_or(ToolError::InvalidManifest)?;
            // Private transfer names cannot alias supplied components or metadata.
            let name = format!("notice-transfer-{index}");
            let path = stage.path.join(&name);
            let mut output = stage.file(&name)?;
            evidence.push(file_evidence(
                &name,
                &output,
                0,
                format!("{:x}", Sha256::digest([])),
            )?);
            self.record_package_staging(
                &stage.path,
                &stage.identity,
                archive_name,
                evidence.clone(),
            )?;
            let mut stream = transport.open(&notice.source_url, checkpoint(cancel, deadline)?)?;
            transfer_to_owned_file(
                &mut *stream,
                &mut output,
                component.bytes,
                &component.sha256,
                cancel,
                deadline,
                |file, bytes, digest| {
                    let last = evidence.last_mut().ok_or(ToolError::InvalidManifest)?;
                    *last = file_evidence(&name, file, bytes, digest)?;
                    self.record_package_staging(
                        &stage.path,
                        &stage.identity,
                        archive_name,
                        evidence.clone(),
                    )
                },
            )?;
            drop(stream);
            drop(output);
            stage.seal_executable(&name)?;
            stage.check()?;
            notices.push(VerifiedNoticeFile {
                component: component.name.clone(),
                path: path.clone(),
                file: native_files::open_regular(&path)?,
            });
        }
        let result = self.install_verified_archive_with_notices_from_file(
            &path, &source, package, cancel, deadline, &notices,
        );
        drop(notices);
        drop(source);
        stage.check()?;
        let stage_path = stage.path.clone();
        drop(stage);
        // Forget only an actually absent original stage; unknown replacements stay recorded.
        if matches!(fs::symlink_metadata(&stage_path),Err(ref e) if e.kind()==io::ErrorKind::NotFound)
        {
            self.forget_staging(&stage_path)?;
        }
        result
    }
}
