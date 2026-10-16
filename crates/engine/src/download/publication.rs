//! Bounded publication journal. Never infer completion from size alone.
use super::*;
use serde::{Deserialize, Serialize};

const MAX_PLAN_BYTES: usize = 128 * 1024;
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    source: PathBuf,
    target: PathBuf,
    bytes: u64,
    #[serde(default, deserialize_with = "stored_identity")]
    identity: Option<crate::file_identity::NativeFileIdentity>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Plan {
    version: u32,
    root: PathBuf,
    entries: Vec<Entry>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    naming: Option<HumanBinding>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct HumanBinding {
    title: String,
    stem: String,
}
fn stored_identity<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<crate::file_identity::NativeFileIdentity>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Stored {
        Legacy((u64, u64)),
        Native(crate::file_identity::NativeFileIdentity),
    }
    Ok(
        Option::<Stored>::deserialize(deserializer)?.map(|value| match value {
            Stored::Legacy((device, inode)) => {
                crate::file_identity::NativeFileIdentity::Unix { device, inode }
            }
            Stored::Native(identity) => identity,
        }),
    )
}
fn identity(path: &Path) -> Option<crate::file_identity::NativeFileIdentity> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let file = fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW)
            .open(path)
            .ok()?;
        crate::file_identity::from_file(&file).ok()
    }
    #[cfg(windows)]
    {
        let file = crate::file_identity::open_regular_no_reparse(path).ok()?;
        // Plans store canonical paths: the opened handle must still name that path.
        if crate::file_identity::final_path(&file).ok()?.as_path() != path {
            return None;
        }
        crate::file_identity::from_file(&file).ok()
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = path;
        None
    }
}
impl Plan {
    pub(super) fn prepare(
        root: &Path,
        id: &str,
        sources: &[PathBuf],
    ) -> Result<Self, DownloadError> {
        Self::prepare_bound(root, id, sources, None)
    }
    pub(super) fn prepare_readable(
        root: &Path,
        id: &str,
        sources: &[PathBuf],
        title: &str,
    ) -> Result<Self, DownloadError> {
        let binding = HumanBinding {
            title: title.into(),
            stem: super::naming::stem(title, id)?,
        };
        Self::prepare_bound(root, id, sources, Some(binding))
    }
    fn prepare_bound(
        root: &Path,
        id: &str,
        sources: &[PathBuf],
        naming: Option<HumanBinding>,
    ) -> Result<Self, DownloadError> {
        if sources.is_empty() || sources.len() > 3 {
            return Err(DownloadError::Policy("publication artifact count"));
        }
        let mut entries: Vec<Entry> = Vec::with_capacity(sources.len());
        for source in sources {
            let observed = fs::symlink_metadata(source)?;
            if !observed.is_file() || observed.file_type().is_symlink() || observed.len() == 0 {
                return Err(DownloadError::Policy("invalid publication source"));
            }
            let name = source
                .file_name()
                .ok_or(DownloadError::Policy("publication filename"))?;
            let target = if let Some(binding) = &naming {
                root.join(super::naming::target_name(
                    &binding.stem,
                    id,
                    name.to_str()
                        .ok_or(DownloadError::Policy("readable filename Unicode source"))?,
                )?)
            } else {
                root.join(name)
            };
            if naming.is_some() && target.to_string_lossy().encode_utf16().count() > 32760 {
                return Err(DownloadError::Policy("readable filename full path limit"));
            }
            if target.try_exists()?
                || entries.iter().any(|entry| {
                    entry.target == target
                        || (naming.is_some()
                            && entry
                                .target
                                .file_name()
                                .and_then(|n| n.to_str())
                                .zip(target.file_name().and_then(|n| n.to_str()))
                                .is_some_and(|(a, b)| a.eq_ignore_ascii_case(b)))
                })
            {
                return Err(DownloadError::Policy("artifact destination already exists"));
            }
            entries.push(Entry {
                source: source.clone(),
                target,
                bytes: observed.len(),
                identity: Some(identity(source).ok_or(DownloadError::Policy(
                    "native publication identity unavailable",
                ))?),
            });
        }
        let plan = Self {
            version: if naming.is_some() { 3 } else { 2 },
            root: root.to_owned(),
            entries,
            naming,
        };
        plan.validate(id)?;
        Ok(plan)
    }
    fn validate(&self, id: &str) -> Result<(), DownloadError> {
        if !matches!(self.version, 1..=3)
            || (self.version == 3) != self.naming.is_some()
            || self
                .naming
                .as_ref()
                .is_some_and(|b| super::naming::stem(&b.title, id).ok().as_ref() != Some(&b.stem))
            || self.entries.is_empty()
            || self.entries.len() > 3
            || !self.root.is_absolute()
            || fs::canonicalize(&self.root)? != self.root
        {
            return Err(DownloadError::Policy("invalid publication plan"));
        }
        let partial = self.root.join(".dlpad-partials").join(id);
        if fs::canonicalize(&partial)? != partial {
            return Err(DownloadError::Policy("publication staging alias"));
        }
        for (index, entry) in self.entries.iter().enumerate() {
            let human_name = if let Some(binding) = &self.naming {
                Some(super::naming::target_name(
                    &binding.stem,
                    id,
                    entry
                        .source
                        .file_name()
                        .and_then(|n| n.to_str())
                        .ok_or(DownloadError::Policy("readable filename source"))?,
                )?)
            } else {
                None
            };
            let valid_name = entry
                .target
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|name| {
                    (if let Some(expected) = &human_name {
                        name == expected
                    } else {
                        name.starts_with(&format!("{id}."))
                    }) && !name.ends_with(".part")
                        && !name.chars().any(char::is_control)
                });
            if (self.version == 3 && entry.identity.is_none())
                || entry.bytes == 0
                || (self.naming.is_some()
                    && entry.target.to_string_lossy().encode_utf16().count() > 32760)
                || entry.target.parent() != Some(self.root.as_path())
                || entry.source.parent() != Some(partial.as_path())
                || (self.naming.is_none() && entry.source.file_name() != entry.target.file_name())
                || !valid_name
                || self.entries[..index].iter().any(|old| {
                    old.target == entry.target
                        || (self.naming.is_some()
                            && old
                                .target
                                .file_name()
                                .and_then(|n| n.to_str())
                                .zip(entry.target.file_name().and_then(|n| n.to_str()))
                                .is_some_and(|(a, b)| a.eq_ignore_ascii_case(b)))
                })
            {
                return Err(DownloadError::Policy("publication path evidence"));
            }
        }
        Ok(())
    }
    pub(super) fn first_target(&self) -> &Path {
        &self.entries[0].target
    }
    pub(super) fn json(&self) -> Result<String, DownloadError> {
        let json = serde_json::to_string(self)?;
        if json.len() > MAX_PLAN_BYTES {
            return Err(DownloadError::Policy("publication plan limit"));
        }
        Ok(json)
    }
    pub(super) fn from_json(json: &str, id: &str) -> Result<Self, DownloadError> {
        if json.len() > MAX_PLAN_BYTES {
            return Err(DownloadError::Policy("publication plan limit"));
        }
        let plan: Self = serde_json::from_str(json)?;
        plan.validate(id)?;
        Ok(plan)
    }
    /// Publish without overwriting. Recovery also requires persisted native identity.
    pub(super) fn publish(
        &self,
        id: &str,
        recovery: bool,
    ) -> Result<Vec<VerifiedArtifact>, DownloadError> {
        self.validate(id)?;
        // Validate the entire staging set before creating any new final link.
        for entry in &self.entries {
            let metadata = fs::symlink_metadata(&entry.source)?;
            if !metadata.is_file()
                || metadata.file_type().is_symlink()
                || metadata.len() != entry.bytes
                || entry.identity.is_none()
                || identity(&entry.source) != entry.identity
            {
                return Err(DownloadError::Policy("publication source changed"));
            }
            match fs::symlink_metadata(&entry.target) {
                Ok(target)
                    if recovery
                        && target.is_file()
                        && !target.file_type().is_symlink()
                        && entry.identity.is_some()
                        && identity(&entry.target) == entry.identity
                        && target.len() == entry.bytes => {}
                Ok(_) => return Err(DownloadError::Policy("publication final collision")),
                Err(error) if error.kind() == io::ErrorKind::NotFound => (),
                Err(error) => return Err(error.into()),
            }
        }
        let mut artifacts = Vec::with_capacity(self.entries.len());
        for entry in &self.entries {
            if !entry.target.try_exists()? {
                fs::hard_link(&entry.source, &entry.target)?;
            }
            // Recheck identity after publication; a raced replacement is not completion.
            let source = fs::symlink_metadata(&entry.source)?;
            let target = fs::symlink_metadata(&entry.target)?;
            if source.file_type().is_symlink()
                || target.file_type().is_symlink()
                || !source.is_file()
                || !target.is_file()
                || source.len() != entry.bytes
                || target.len() != entry.bytes
                || identity(&entry.source) != entry.identity
                || identity(&entry.target) != entry.identity
            {
                return Err(DownloadError::Policy("publication identity changed"));
            }
            artifacts.push(VerifiedArtifact {
                path: entry.target.clone(),
                bytes: entry.bytes,
            });
        }
        Ok(artifacts)
    }
    pub(super) fn cleanup(&self) {
        for entry in &self.entries {
            if let (Ok(source), Ok(target)) = (
                fs::symlink_metadata(&entry.source),
                fs::symlink_metadata(&entry.target),
            ) && entry.identity.is_some()
                && identity(&entry.source) == entry.identity
                && identity(&entry.target) == entry.identity
                && !source.file_type().is_symlink()
                && !target.file_type().is_symlink()
            {
                let _ = fs::remove_file(&entry.source);
            }
        }
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    fn fixture() -> (tempfile::TempDir, Vec<PathBuf>) {
        let directory = tempfile::tempdir().unwrap();
        let staging = directory.path().join(".dlpad-partials/job");
        fs::create_dir_all(&staging).unwrap();
        let paths: Vec<_> = ["job.mp4", "job.ko.vtt", "job.en.vtt"]
            .iter()
            .map(|name| staging.join(name))
            .collect();
        for path in &paths {
            fs::write(path, b"synthetic").unwrap();
        }
        (directory, paths)
    }
    #[test]
    fn partial_batch_crash_is_recovered_using_persisted_identity() {
        let (directory, paths) = fixture();
        let plan = Plan::prepare(directory.path(), "job", &paths).unwrap();
        let json = plan.json().unwrap();
        fs::hard_link(&paths[0], directory.path().join("job.mp4")).unwrap();
        let recovered = Plan::from_json(&json, "job").unwrap();
        assert_eq!(recovered.publish("job", true).unwrap().len(), 3);
        assert_eq!(recovered.publish("job", true).unwrap().len(), 3);
        recovered.cleanup();
        for path in paths {
            assert!(!path.exists());
        }
        assert!(directory.path().join("job.ko.vtt").is_file());
    }
    #[test]
    fn missing_identity_never_authorizes_fresh_publication() {
        let (directory, paths) = fixture();
        let plan = Plan::prepare(directory.path(), "job", &paths).unwrap();
        let mut value = serde_json::to_value(&plan).unwrap();
        for entry in value["entries"].as_array_mut().unwrap() {
            entry["identity"] = serde_json::Value::Null;
        }
        let unverified = Plan::from_json(&value.to_string(), "job").unwrap();
        assert!(unverified.publish("job", false).is_err());
        assert!(unverified.publish("job", true).is_err());
        assert!(!directory.path().join("job.mp4").exists());
        assert!(paths.iter().all(|path| path.is_file()));
    }
    #[test]
    fn legacy_unix_tuple_plan_remains_recoverable() {
        let (directory, paths) = fixture();
        let plan = Plan::prepare(directory.path(), "job", &paths).unwrap();
        let mut value = serde_json::to_value(&plan).unwrap();
        value["version"] = 1.into();
        for entry in value["entries"].as_array_mut().unwrap() {
            let identity = &entry["identity"]["Unix"];
            entry["identity"] = serde_json::json!([identity["device"], identity["inode"]]);
        }
        let migrated = Plan::from_json(&value.to_string(), "job").unwrap();
        assert_eq!(migrated.publish("job", true).unwrap().len(), 3);
    }
    #[test]
    fn replacement_or_foreign_target_never_completes_or_overwrites() {
        for replace_source in [false, true] {
            let (directory, paths) = fixture();
            let plan = Plan::prepare(directory.path(), "job", &paths).unwrap();
            if replace_source {
                fs::rename(&paths[1], paths[1].with_extension("saved")).unwrap();
                fs::write(&paths[1], b"synthetic").unwrap();
            } else {
                fs::write(directory.path().join("job.ko.vtt"), b"user-file").unwrap();
            }
            assert!(plan.publish("job", true).is_err());
            assert!(!directory.path().join("job.mp4").exists());
            assert!(paths[0].exists());
        }
    }
}

#[cfg(all(test, windows))]
mod windows_tests {
    use super::*;
    #[test]
    fn native_windows_partial_publication_recovers_and_replacement_is_rejected() {
        for replaced in [false, true] {
            let directory = tempfile::tempdir().unwrap();
            let root = fs::canonicalize(directory.path()).unwrap();
            let staging = root.join(".dlpad-partials/job");
            fs::create_dir_all(&staging).unwrap();
            let video = staging.join("job.mp4");
            let subtitle = staging.join("job.ko.vtt");
            fs::write(&video, b"synthetic").unwrap();
            fs::write(&subtitle, b"WEBVTT").unwrap();
            let plan = Plan::prepare(&root, "job", &[video.clone(), subtitle.clone()]).unwrap();
            assert!(plan.entries.iter().all(|entry| matches!(
                entry.identity,
                Some(crate::file_identity::NativeFileIdentity::Windows { .. })
            )));
            let json = plan.json().unwrap();
            fs::hard_link(&video, root.join("job.mp4")).unwrap();
            if replaced {
                fs::rename(&subtitle, staging.join("old.vtt")).unwrap();
                fs::write(&subtitle, b"WEBVTT").unwrap();
            }
            let recovered = Plan::from_json(&json, "job").unwrap();
            if replaced {
                assert!(recovered.publish("job", true).is_err());
                assert!(!root.join("job.ko.vtt").exists());
                assert!(subtitle.exists());
            } else {
                assert_eq!(recovered.publish("job", true).unwrap().len(), 2);
                assert_eq!(recovered.publish("job", true).unwrap().len(), 2);
                recovered.cleanup();
                assert!(!subtitle.exists());
                assert_eq!(fs::read(root.join("job.ko.vtt")).unwrap(), b"WEBVTT");
            }
        }
    }
}

#[cfg(test)]
mod readable_tests {
    use super::*;
    fn fixture() -> (tempfile::TempDir, PathBuf, Vec<PathBuf>) {
        let dir = tempfile::tempdir().unwrap();
        let root = fs::canonicalize(dir.path()).unwrap();
        let partial = root.join(".dlpad-partials/job");
        fs::create_dir_all(&partial).unwrap();
        let sources = ["job.mp4", "job.ko.vtt", "job.en.ttml"]
            .map(|name| partial.join(name))
            .to_vec();
        for source in &sources {
            fs::write(source, b"synthetic").unwrap();
        }
        (dir, root, sources)
    }
    #[test]
    fn readable_batch_recovers_partial_links_using_exact_persisted_identity() {
        let (_dir, root, sources) = fixture();
        let plan = Plan::prepare_readable(&root, "job", &sources, "한글 강의 🎬").unwrap();
        let json = plan.json().unwrap();
        assert_eq!(plan.version, 3);
        fs::hard_link(&sources[0], plan.first_target()).unwrap();
        let restored = Plan::from_json(&json, "job").unwrap();
        let artifacts = restored.publish("job", true).unwrap();
        assert_eq!(artifacts.len(), 3);
        assert_eq!(
            artifacts[0].path.file_name().unwrap(),
            "한글 강의 🎬 [job].mp4"
        );
        assert_eq!(
            artifacts[1].path.file_name().unwrap(),
            "한글 강의 🎬 [job].ko.vtt"
        );
        assert_eq!(
            artifacts[2].path.file_name().unwrap(),
            "한글 강의 🎬 [job].en.ttml"
        );
        assert_eq!(restored.publish("job", true).unwrap().len(), 3);
        restored.cleanup();
        assert!(sources.iter().all(|p| !p.exists()));
        assert_eq!(fs::read(&artifacts[1].path).unwrap(), b"synthetic");
    }
    #[test]
    fn readable_collision_and_replaced_source_never_publish_or_overwrite() {
        for replace_source in [false, true] {
            let (_dir, root, sources) = fixture();
            let plan = Plan::prepare_readable(&root, "job", &sources, "강의").unwrap();
            if replace_source {
                fs::rename(&sources[1], root.join("retained-source")).unwrap();
                fs::write(&sources[1], b"synthetic").unwrap();
            } else {
                fs::write(&plan.entries[1].target, b"user data").unwrap();
            }
            assert!(plan.publish("job", true).is_err());
            assert!(!plan.entries[0].target.exists());
            assert!(sources[0].exists());
            if !replace_source {
                assert_eq!(fs::read(&plan.entries[1].target).unwrap(), b"user data");
            }
        }
        let (_dir, root, sources) = fixture();
        fs::write(root.join("강의 [job].mp4"), b"preserve").unwrap();
        assert!(Plan::prepare_readable(&root, "job", &sources, "강의").is_err());
        assert_eq!(fs::read(root.join("강의 [job].mp4")).unwrap(), b"preserve");
    }
    #[test]
    fn readable_journal_binding_corruption_and_legacy_relabeling_are_refused() {
        let (_dir, root, sources) = fixture();
        let plan = Plan::prepare_readable(&root, "job", &sources, "강의").unwrap();
        let value = serde_json::to_value(&plan).unwrap();
        for field in [
            "title", "stem", "target", "source", "identity", "version", "naming",
        ] {
            let mut bad = value.clone();
            match field {
                "title" => bad["naming"]["title"] = "other".into(),
                "stem" => bad["naming"]["stem"] = "arbitrary [job]".into(),
                "target" => {
                    bad["entries"][0]["target"] = root.join("outside.mp4").to_str().unwrap().into()
                }
                "source" => {
                    bad["entries"][0]["source"] = root.join("foreign.mp4").to_str().unwrap().into()
                }
                "identity" => bad["entries"][0]["identity"] = serde_json::Value::Null,
                "version" => bad["version"] = 2.into(),
                _ => bad["naming"] = serde_json::Value::Null,
            };
            assert!(Plan::from_json(&bad.to_string(), "job").is_err());
        }
        let legacy = Plan::prepare(&root, "job", &sources).unwrap();
        assert_eq!(legacy.version, 2);
        let json = legacy.json().unwrap();
        assert!(!json.contains("naming"));
        assert!(Plan::from_json(&json, "job").is_ok());
        assert!(Plan::from_json(&json, "other-job").is_err());
    }
    #[test]
    fn source_case_alias_and_unknown_suffix_are_rejected_before_linking() {
        let (_dir, root, sources) = fixture();
        let alias = sources[0].with_file_name("job.MP4");
        #[cfg(not(windows))]
        fs::write(&alias, b"synthetic").unwrap();
        assert!(
            Plan::prepare_readable(&root, "job", &[sources[0].clone(), alias], "강의").is_err()
        );
        let bad = sources[0].with_file_name("job.en.ass");
        fs::write(&bad, b"synthetic").unwrap();
        assert!(Plan::prepare_readable(&root, "job", &[bad], "강의").is_err());
        assert!(!root.join("강의 [job].mp4").exists());
    }
    #[cfg(windows)]
    #[test]
    fn readable_target_under_owned_long_root_keeps_bounded_hangul_component() {
        let dir = tempfile::tempdir().unwrap();
        let mut root = fs::canonicalize(dir.path()).unwrap();
        for n in 0..18 {
            root = root.join(format!("owned-long-directory-{n:02}"));
        }
        fs::create_dir_all(&root).unwrap();
        let partial = root.join(".dlpad-partials/job");
        fs::create_dir_all(&partial).unwrap();
        let source = partial.join("job.mp4");
        fs::write(&source, b"synthetic").unwrap();
        let plan = Plan::prepare_readable(&root, "job", &[source], &"한글🎬".repeat(200)).unwrap();
        assert!(plan.first_target().to_string_lossy().encode_utf16().count() > 260);
        assert!(
            plan.first_target()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .encode_utf16()
                .count()
                <= 240
        );
        let artifacts = plan.publish("job", false).unwrap();
        assert_eq!(fs::read(&artifacts[0].path).unwrap(), b"synthetic");
    }
}
