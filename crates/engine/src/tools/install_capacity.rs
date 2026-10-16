//! Complete dedicated-root inventory bound, shared by new package installations.
use super::*;
impl ToolManager {
    pub(super) fn verify_install_inventory_capacity(&self) -> Result<(), ToolError> {
        self.verify_root()?;
        let mut entries = 0usize;
        let mut installations = 0usize;
        let mut pending = vec![(self.root.clone(), 0usize)];
        while let Some((path, depth)) = pending.pop() {
            let directory = native_files::open_directory(&path)?;
            for entry in fs::read_dir(&path)? {
                entries += 1;
                if entries > 2048 {
                    return Err(ToolError::InventoryLimit);
                }
                let entry = entry?;
                let name = entry.file_name();
                let name = name.to_str().ok_or(ToolError::UnsafePath)?;
                let meta = fs::symlink_metadata(entry.path())?;
                if depth == 0
                    && ((name.starts_with(".staging-") && identifier(name))
                        || matches!(
                            name,
                            ".dlpad-owner"
                                | ".install-transfer-lock"
                                | "activation.sqlite"
                                | "activation.sqlite-journal"
                                | "activation.sqlite-wal"
                                | "activation.sqlite-shm"
                        ))
                {
                    continue;
                }
                safe_components(&entry.path())?;
                if depth < 3 {
                    if !identifier(name) || !meta.is_dir() {
                        return Err(ToolError::UnexpectedFiles);
                    }
                    if depth == 2 {
                        installations += 1;
                        if installations >= 128 {
                            return Err(ToolError::InventoryLimit);
                        }
                    }
                    pending.push((entry.path(), depth + 1));
                } else if !meta.is_file() {
                    return Err(ToolError::UnexpectedFiles);
                }
            }
            if !native_files::same_directory(&path, &directory) {
                return Err(ToolError::UnsafePath);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn capacity_counts_all_tools_and_preserves_unknown_data() {
        let root = tempfile::tempdir().unwrap();
        let manager = ToolManager::new(&root.path().join("managed")).unwrap();
        for n in 0..127 {
            fs::create_dir_all(manager.root.join(format!("tool{n}/1/native"))).unwrap();
        }
        assert!(manager.verify_install_inventory_capacity().is_ok());
        fs::create_dir_all(manager.root.join("last/1/native")).unwrap();
        assert!(matches!(
            manager.verify_install_inventory_capacity(),
            Err(ToolError::InventoryLimit)
        ));
        fs::write(root.path().join("foreign-sentinel"), b"preserve").unwrap();
        assert_eq!(
            fs::read(root.path().join("foreign-sentinel")).unwrap(),
            b"preserve"
        );
    }
    #[test]
    fn shared_root_lock_prevents_capacity_reservation_races() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("managed");
        let first = ToolManager::new(&path).unwrap();
        let second = ToolManager::new(&path).unwrap();
        let lock = first.acquire_install_lock().unwrap();
        assert!(matches!(
            second.acquire_install_lock(),
            Err(ToolError::Io(_))
        ));
        drop(lock);
        assert!(second.acquire_install_lock().is_ok());
    }
    #[cfg(unix)]
    #[test]
    fn original_manager_root_cannot_be_impersonated_by_copied_marker() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("managed");
        let manager = ToolManager::new(&path).unwrap();
        fs::rename(&path, temp.path().join("original")).unwrap();
        fs::create_dir(&path).unwrap();
        fs::write(path.join(".dlpad-owner"), OWNER).unwrap();
        fs::write(path.join("foreign"), b"preserve").unwrap();
        assert!(matches!(manager.verify_root(), Err(ToolError::UnsafePath)));
        assert_eq!(fs::read(path.join("foreign")).unwrap(), b"preserve");
    }
}
