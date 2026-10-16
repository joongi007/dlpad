//! Bounded uncompressed TAR reader. Compressed archive trust belongs to the caller.
use super::{ArchiveComponent, ToolError};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs::File,
    io::{Read, Seek, Write},
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};

pub struct TarComponentMapping {
    pub member: String,
    pub component: ArchiveComponent,
}
pub struct TarLimits {
    pub max_entries: usize,
    pub max_total_bytes: u64,
    pub max_selected_bytes: u64,
    pub max_stream_bytes: u64,
}
impl TarLimits {
    fn validate(&self) -> Result<(), ToolError> {
        if self.max_entries == 0
            || self.max_entries > 4096
            || self.max_total_bytes == 0
            || self.max_total_bytes > 768 * 1024 * 1024
            || self.max_selected_bytes == 0
            || self.max_selected_bytes > 384 * 1024 * 1024
            || self.max_stream_bytes == 0
            || self.max_stream_bytes > 800 * 1024 * 1024
        {
            return Err(ToolError::DownloadLimit);
        }
        Ok(())
    }
}
fn check(cancel: &AtomicBool, deadline: Instant) -> Result<(), ToolError> {
    if cancel.load(Ordering::Acquire) {
        return Err(ToolError::Cancelled);
    }
    if Instant::now() >= deadline {
        return Err(ToolError::TimedOut);
    }
    Ok(())
}
fn field(bytes: &[u8]) -> Result<&str, ToolError> {
    let end = bytes.iter().position(|v| *v == 0).unwrap_or(bytes.len());
    if bytes[end..].iter().any(|v| *v != 0) {
        return Err(ToolError::InvalidManifest);
    }
    std::str::from_utf8(&bytes[..end]).map_err(|_| ToolError::InvalidManifest)
}
fn octal(bytes: &[u8]) -> Result<u64, ToolError> {
    let value = std::str::from_utf8(bytes)
        .map_err(|_| ToolError::InvalidManifest)?
        .trim_matches(['\0', ' ']);
    if value.is_empty() || !value.bytes().all(|b| (b'0'..=b'7').contains(&b)) {
        return Err(ToolError::InvalidManifest);
    }
    u64::from_str_radix(value, 8).map_err(|_| ToolError::InvalidManifest)
}
fn path(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 255
        && value.is_ascii()
        && !value.contains('\\')
        && value.split('/').all(|part| {
            !part.is_empty()
                && part != "."
                && part != ".."
                && part.bytes().all(|b| b.is_ascii_graphic())
        })
}
fn read_some<R: Read>(
    source: &mut R,
    buf: &mut [u8],
    used: &mut u64,
    limits: &TarLimits,
    cancel: &AtomicBool,
    deadline: Instant,
) -> Result<usize, ToolError> {
    check(cancel, deadline)?;
    let n = source.read(buf)?;
    *used = used.checked_add(n as u64).ok_or(ToolError::DownloadLimit)?;
    if *used > limits.max_stream_bytes {
        return Err(ToolError::DownloadLimit);
    }
    Ok(n)
}
fn exact<R: Read>(
    source: &mut R,
    mut buf: &mut [u8],
    used: &mut u64,
    limits: &TarLimits,
    cancel: &AtomicBool,
    deadline: Instant,
) -> Result<(), ToolError> {
    while !buf.is_empty() {
        let n = read_some(source, buf, used, limits, cancel, deadline)?;
        if n == 0 {
            return Err(ToolError::UnexpectedSize);
        }
        buf = &mut buf[n..];
    }
    Ok(())
}
/// Writes one declared component; validates the entire remaining stream before success.
/// Failure may leave partial output. Caller owns its cleanup and compressed source pin.
pub struct TarExtractionRequest<'a> {
    pub root: &'a str,
    pub mappings: &'a [TarComponentMapping],
    pub selected: &'a str,
    pub limits: &'a TarLimits,
}
pub fn extract_tar_component<R: Read>(
    source: &mut R,
    request: TarExtractionRequest<'_>,
    output: &mut File,
    cancel: &AtomicBool,
    deadline: Instant,
) -> Result<u64, ToolError> {
    let TarExtractionRequest {
        root,
        mappings,
        selected,
        limits,
    } = request;
    limits.validate()?;
    check(cancel, deadline)?;
    if !path(root)
        || root.contains('/')
        || mappings.is_empty()
        || mappings.len() > 8
        || output.metadata()?.len() != 0
        || output.stream_position()? != 0
    {
        return Err(ToolError::InvalidManifest);
    }
    let mut map_names = BTreeSet::new();
    let mut flat_names = BTreeSet::new();
    for mapping in mappings {
        let c = &mapping.component;
        if !path(&mapping.member)
            || !mapping.member.starts_with(&format!("{root}/"))
            || !map_names.insert(&mapping.member)
            || !super::archive_manifest::component_name(&c.name)
            || !flat_names.insert(c.name.to_ascii_lowercase())
            || c.bytes == 0
            || c.bytes > limits.max_selected_bytes
            || c.sha256.len() != 64
            || !c
                .sha256
                .bytes()
                .all(|v| v.is_ascii_hexdigit() && !v.is_ascii_uppercase())
        {
            return Err(ToolError::InvalidManifest);
        }
    }
    let wanted = mappings
        .iter()
        .position(|m| m.component.name == selected)
        .ok_or(ToolError::InvalidManifest)?;
    let mut found = BTreeSet::new();
    let mut names = BTreeSet::new();
    let mut used = 0;
    let mut total = 0u64;
    let mut count = 0usize;
    let mut selected_bytes = 0;
    let mut sha = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        let mut header = [0u8; 512];
        exact(source, &mut header, &mut used, limits, cancel, deadline)?;
        if header.iter().all(|b| *b == 0) {
            exact(source, &mut header, &mut used, limits, cancel, deadline)?;
            if header.iter().any(|b| *b != 0) {
                return Err(ToolError::InvalidManifest);
            }
            loop {
                let n = read_some(source, &mut buffer, &mut used, limits, cancel, deadline)?;
                if n == 0 {
                    break;
                }
                if buffer[..n].iter().any(|b| *b != 0) {
                    return Err(ToolError::InvalidManifest);
                }
            }
            if used % 512 != 0 {
                return Err(ToolError::UnexpectedSize);
            }
            break;
        }
        count += 1;
        if count > limits.max_entries {
            return Err(ToolError::DownloadLimit);
        }
        let checksum = octal(&header[148..156])?;
        let actual: u64 = header
            .iter()
            .enumerate()
            .map(|(i, b)| {
                if (148..156).contains(&i) {
                    32
                } else {
                    u64::from(*b)
                }
            })
            .sum();
        if checksum != actual {
            return Err(ToolError::InvalidManifest);
        }
        let gnu = &header[257..265] == b"ustar  \0";
        if !gnu && (&header[257..263] != b"ustar\0" || &header[263..265] != b"00") {
            return Err(ToolError::InvalidManifest);
        }
        if gnu && header[345..512].iter().any(|b| *b != 0) {
            return Err(ToolError::InvalidManifest);
        }
        let name = field(&header[..100])?;
        let prefix = if gnu { "" } else { field(&header[345..500])? };
        let mut member = if prefix.is_empty() {
            name.to_owned()
        } else {
            format!("{prefix}/{name}")
        };
        let directory = header[156] == b'5';
        if directory {
            member = member.trim_end_matches('/').to_owned();
        }
        if !path(&member)
            || (member != root && !member.starts_with(&format!("{root}/")))
            || !names.insert(member.clone())
        {
            return Err(ToolError::UnsafePath);
        }
        if ![0, b'0', b'5'].contains(&header[156]) || !field(&header[157..257])?.is_empty() {
            return Err(ToolError::UnsafePath);
        }
        let bytes = octal(&header[124..136])?;
        if directory && bytes != 0 {
            return Err(ToolError::InvalidManifest);
        }
        total = total.checked_add(bytes).ok_or(ToolError::DownloadLimit)?;
        if total > limits.max_total_bytes {
            return Err(ToolError::DownloadLimit);
        }
        let index = mappings.iter().position(|m| m.member == member);
        if let Some(i) = index {
            if directory || bytes != mappings[i].component.bytes {
                return Err(ToolError::UnexpectedSize);
            }
            found.insert(i);
        }
        let mut remaining = bytes;
        while remaining > 0 {
            let n = remaining.min(buffer.len() as u64) as usize;
            exact(
                source,
                &mut buffer[..n],
                &mut used,
                limits,
                cancel,
                deadline,
            )?;
            if index == Some(wanted) {
                output.write_all(&buffer[..n])?;
                sha.update(&buffer[..n]);
                selected_bytes += n as u64;
            }
            remaining -= n as u64;
        }
        let padding = ((512 - bytes % 512) % 512) as usize;
        exact(
            source,
            &mut buffer[..padding],
            &mut used,
            limits,
            cancel,
            deadline,
        )?;
        if buffer[..padding].iter().any(|b| *b != 0) {
            return Err(ToolError::InvalidManifest);
        }
    }
    if found.len() != mappings.len() || selected_bytes != mappings[wanted].component.bytes {
        return Err(ToolError::UnexpectedSize);
    }
    if format!("{:x}", sha.finalize()) != mappings[wanted].component.sha256 {
        return Err(ToolError::HashMismatch);
    }
    check(cancel, deadline)?;
    Ok(selected_bytes)
}

#[cfg(test)]
mod tests {
    use super::super::ArchiveComponentRole;
    use super::*;
    use std::{
        io::{Cursor, SeekFrom},
        sync::atomic::AtomicU64,
        time::Duration,
    };
    static SEQ: AtomicU64 = AtomicU64::new(0);
    fn header(name: &str, bytes: u64, kind: u8) -> [u8; 512] {
        let mut h = [0; 512];
        h[..name.len()].copy_from_slice(name.as_bytes());
        h[124..136].copy_from_slice(format!("{bytes:011o}\0").as_bytes());
        h[156] = kind;
        h[257..265].copy_from_slice(b"ustar\x0000");
        sum(&mut h);
        h
    }
    fn sum(h: &mut [u8; 512]) {
        h[148..156].fill(b' ');
        let sum: u64 = h.iter().map(|b| u64::from(*b)).sum();
        h[148..156].copy_from_slice(format!("{sum:06o}\0 ").as_bytes());
    }
    fn archive() -> Vec<u8> {
        let mut v = header("root/", 0, b'5').to_vec();
        v.extend(header("root/bin/tool", 3, b'0'));
        v.extend(b"abc");
        v.resize(1536, 0);
        v.resize(2560, 0);
        v
    }
    fn mapping() -> Vec<TarComponentMapping> {
        vec![TarComponentMapping {
            member: "root/bin/tool".into(),
            component: ArchiveComponent {
                name: "tool".into(),
                bytes: 3,
                sha256: format!("{:x}", Sha256::digest(b"abc")),
                role: ArchiveComponentRole::Executable,
            },
        }]
    }
    fn run(
        v: Vec<u8>,
        maps: &[TarComponentMapping],
        cancel: bool,
        expired: bool,
        limit: usize,
    ) -> Result<u64, ToolError> {
        let name = std::env::temp_dir().join(format!(
            "dlpad-tar-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        let mut file = std::fs::OpenOptions::new()
            .create_new(true)
            .read(true)
            .write(true)
            .open(&name)
            .unwrap();
        let result = extract_tar_component(
            &mut Cursor::new(v),
            TarExtractionRequest {
                root: "root",
                mappings: maps,
                selected: "tool",
                limits: &TarLimits {
                    max_entries: limit,
                    max_total_bytes: 1024,
                    max_selected_bytes: 512,
                    max_stream_bytes: 8192,
                },
            },
            &mut file,
            &AtomicBool::new(cancel),
            if expired {
                Instant::now()
            } else {
                Instant::now() + Duration::from_secs(3)
            },
        );
        if result.is_ok() {
            file.seek(SeekFrom::Start(0)).unwrap();
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes).unwrap();
            assert_eq!(bytes, b"abc");
        }
        drop(file);
        std::fs::remove_file(name).unwrap();
        result
    }
    #[test]
    fn regular_and_gnu_success() {
        assert_eq!(run(archive(), &mapping(), false, false, 8).unwrap(), 3);
        let mut v = archive();
        for pos in [0, 512] {
            let mut h: [u8; 512] = v[pos..pos + 512].try_into().unwrap();
            h[257..265].copy_from_slice(b"ustar  \0");
            sum(&mut h);
            v[pos..pos + 512].copy_from_slice(&h);
        }
        assert!(run(v, &mapping(), false, false, 8).is_ok());
    }
    #[test]
    fn checksum_paths_types_duplicates() {
        let mut v = archive();
        v[0] ^= 1;
        assert!(run(v, &mapping(), false, false, 8).is_err());
        for name in ["../tool", "/tool", "root/../tool", "root\\tool"] {
            let mut v = archive();
            v[512..1024].copy_from_slice(&header(name, 3, b'0'));
            assert!(run(v, &mapping(), false, false, 8).is_err());
        }
        for kind in [b'1', b'2', b'3', b'6', b'L', b'x', b'S'] {
            let mut v = archive();
            v[512..1024].copy_from_slice(&header("root/bin/tool", 3, kind));
            assert!(run(v, &mapping(), false, false, 8).is_err());
        }
        let mut v = archive();
        let entry = v[512..1536].to_vec();
        v.splice(1536..1536, entry);
        assert!(run(v, &mapping(), false, false, 8).is_err());
    }
    #[test]
    fn truncated_padding_tail_and_hash() {
        for length in [0, 511, 1025, 1536, 2048] {
            let mut v = archive();
            v.truncate(length);
            assert!(run(v, &mapping(), false, false, 8).is_err());
        }
        let mut v = archive();
        v[1027] = 1;
        assert!(run(v, &mapping(), false, false, 8).is_err());
        let mut v = archive();
        v.extend([1; 512]);
        assert!(run(v, &mapping(), false, false, 8).is_err());
        let mut maps = mapping();
        maps[0].component.sha256 = "0".repeat(64);
        assert!(matches!(
            run(archive(), &maps, false, false, 8),
            Err(ToolError::HashMismatch)
        ));
    }
    #[test]
    fn cancellation_deadline_entrylimit_and_manifest() {
        assert!(matches!(
            run(archive(), &mapping(), true, false, 8),
            Err(ToolError::Cancelled)
        ));
        assert!(matches!(
            run(archive(), &mapping(), false, true, 8),
            Err(ToolError::TimedOut)
        ));
        assert!(matches!(
            run(archive(), &mapping(), false, false, 1),
            Err(ToolError::DownloadLimit)
        ));
        let mut v = archive();
        let mut h = header("root/bin/tool", 2048, b'0');
        sum(&mut h);
        v[512..1024].copy_from_slice(&h);
        assert!(matches!(
            run(v, &mapping(), false, false, 8),
            Err(ToolError::DownloadLimit)
        ));
        let mut maps = mapping();
        maps[0].member = "root/no-tool".into();
        assert!(matches!(
            run(archive(), &maps, false, false, 8),
            Err(ToolError::UnexpectedSize)
        ));
    }
}
