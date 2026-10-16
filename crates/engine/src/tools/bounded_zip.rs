//! Capability-only ZIP extraction; caller owns publication and partial-output cleanup.
use super::{ArchiveLayout, ArchivePackage, MAX_ARCHIVE_BYTES, MAX_EXPANDED_BYTES, ToolError};
use rawzip::{CompressionMethod, ZipArchive};
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};
const BUFFER: usize = 64 * 1024;
const MAX_ENTRIES: usize = 32;
const MAPPED_ENTRIES: usize = 4096;
const MAPPED_TOTAL: u64 = 768 * 1024 * 1024;
const MAPPED_SELECTED: u64 = 384 * 1024 * 1024;
fn check(cancel: &AtomicBool, deadline: Instant) -> Result<(), ToolError> {
    if cancel.load(Ordering::Acquire) {
        return Err(ToolError::Cancelled);
    }
    if Instant::now() >= deadline {
        return Err(ToolError::TimedOut);
    }
    Ok(())
}
enum Decoded<R: Read> {
    Stored(R),
    Deflate(flate2::read::DeflateDecoder<R>),
}
impl<R: Read> Read for Decoded<R> {
    fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::Stored(reader) => reader.read(out),
            Self::Deflate(reader) => reader.read(out),
        }
    }
}
/// Hash the complete archive before parsing; never executes or selects output paths.
pub fn extract_verified_zip(
    source: &File,
    package: &ArchivePackage,
    component_name: &str,
    output: &mut File,
    cancel: &AtomicBool,
    deadline: Instant,
) -> Result<u64, ToolError> {
    check(cancel, deadline)?;
    package.validate()?;
    let mapped = match (package.version, &package.layout) {
        (1 | 3, None) => None,
        (2, Some(ArchiveLayout::Zip { root, mappings })) => Some((root, mappings)),
        _ => return Err(ToolError::InvalidManifest),
    };
    let entry_limit = if mapped.is_some() {
        MAPPED_ENTRIES
    } else {
        MAX_ENTRIES
    };
    let expanded_limit = if mapped.is_some() {
        MAPPED_TOTAL
    } else {
        MAX_EXPANDED_BYTES
    };
    let selected_limit = if mapped.is_some() {
        MAPPED_SELECTED
    } else {
        MAX_EXPANDED_BYTES
    };
    let selected = package
        .archive_components()
        .find(|c| c.name == component_name)
        .ok_or(ToolError::InvalidManifest)?;
    if output.metadata()?.len() != 0 || output.stream_position()? != 0 {
        return Err(ToolError::UnsafePath);
    }
    if !source.metadata()?.is_file()
        || source.metadata()?.len() != package.archive_bytes
        || package.archive_bytes > MAX_ARCHIVE_BYTES
    {
        return Err(ToolError::UnexpectedSize);
    }
    let mut cursor = source;
    cursor.seek(SeekFrom::Start(0))?;
    let mut buffer = [0u8; BUFFER];
    let mut hash = Sha256::new();
    let mut bytes = 0u64;
    loop {
        check(cancel, deadline)?;
        let n = cursor.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        bytes = bytes
            .checked_add(n as u64)
            .ok_or(ToolError::DownloadLimit)?;
        if bytes > package.archive_bytes {
            return Err(ToolError::UnexpectedSize);
        }
        hash.update(&buffer[..n]);
    }
    if bytes != package.archive_bytes {
        return Err(ToolError::UnexpectedSize);
    }
    if format!("{:x}", hash.finalize()) != package.archive_sha256.to_ascii_lowercase() {
        return Err(ToolError::HashMismatch);
    }
    let archive = ZipArchive::from_file(source.try_clone()?, &mut buffer)
        .map_err(|_| ToolError::InvalidManifest)?;
    if archive.entries_hint() == 0 || archive.entries_hint() > entry_limit as u64 {
        return Err(ToolError::DownloadLimit);
    }
    let mut entries = archive.entries(&mut buffer);
    let mut local_buffer = [0u8; BUFFER];
    let mut names = Vec::with_capacity(entry_limit);
    let mut ranges: Vec<(u64, u64)> = Vec::with_capacity(entry_limit);
    let mut target = None;
    let mut expanded = 0u64;
    while let Some(header) = entries
        .next_entry()
        .map_err(|_| ToolError::InvalidManifest)?
    {
        check(cancel, deadline)?;
        if names.len() == entry_limit {
            return Err(ToolError::DownloadLimit);
        }
        let path = header.file_path();
        let raw = path.as_ref();
        let name = std::str::from_utf8(raw).map_err(|_| ToolError::UnsafePath)?;
        let is_dir = header.is_dir();
        let component = if let Some((root, mappings)) = mapped {
            let normalized = if is_dir {
                name.strip_suffix('/').ok_or(ToolError::UnsafePath)?
            } else {
                name
            };
            if normalized.len() > 255
                || !normalized.is_ascii()
                || normalized.contains('\\')
                || !normalized
                    .split('/')
                    .all(super::archive_manifest::component_name)
                || !(normalized == root || normalized.starts_with(&format!("{root}/")))
                || (!is_dir && normalized == root)
                || (is_dir && header.mode().value() & 0o170000 != 0o040000)
                || (!is_dir && header.mode().value() & 0o170000 != 0o100000)
            {
                return Err(ToolError::UnsafePath);
            }
            mappings
                .iter()
                .find(|m| m.member == name)
                .and_then(|m| package.components.iter().find(|c| c.name == m.component))
        } else {
            if !super::identifier(name) || is_dir || header.mode().value() & 0o170000 != 0o100000 {
                return Err(ToolError::UnsafePath);
            }
            Some(
                package
                    .archive_components()
                    .find(|c| c.name == name)
                    .ok_or(ToolError::InvalidManifest)?,
            )
        };
        // A directory and regular member cannot alias the same case-folded path.
        let normalized = name.trim_end_matches('/');
        if names
            .iter()
            .any(|old: &String| old.trim_end_matches('/').eq_ignore_ascii_case(normalized))
        {
            return Err(ToolError::InvalidManifest);
        }
        let folded = normalized.to_ascii_lowercase();
        if mapped.is_some()
            && names.iter().any(|old| {
                let old_folded = old.to_ascii_lowercase();
                (!old.ends_with('/') && folded.starts_with(&format!("{old_folded}/")))
                    || (!is_dir && old_folded.starts_with(&format!("{folded}/")))
            })
        {
            return Err(ToolError::UnsafePath);
        }
        if let Some(component) = component
            && (is_dir || header.uncompressed_size_hint() != component.bytes)
        {
            return Err(ToolError::UnexpectedSize);
        }
        if is_dir && (header.uncompressed_size_hint() != 0 || header.crc32() != 0) {
            return Err(ToolError::UnexpectedSize);
        }
        if header.flags().is_encrypted()
            || header.flags().has_strong_encryption()
            || header.flags().is_masked()
        {
            return Err(ToolError::InvalidManifest);
        }
        let method = header.compression_method();
        if method != CompressionMethod::STORE && method != CompressionMethod::DEFLATE {
            return Err(ToolError::InvalidManifest);
        }
        expanded = expanded
            .checked_add(header.uncompressed_size_hint())
            .ok_or(ToolError::DownloadLimit)?;
        if expanded > expanded_limit {
            return Err(ToolError::DownloadLimit);
        }
        let wayfinder = header.wayfinder();
        let entry = archive
            .get_entry(wayfinder)
            .map_err(|_| ToolError::InvalidManifest)?;
        let local = entry
            .local_header(&mut local_buffer)
            .map_err(|_| ToolError::InvalidManifest)?;
        if local.file_path().as_ref() != raw
            || local.flags().bits() != header.flags().bits()
            || local.compression_method() != method
        {
            return Err(ToolError::InvalidManifest);
        }
        if !header.flags().has_data_descriptor()
            && (local.crc32() != header.crc32()
                || local.compressed_size_hint() != header.compressed_size_hint()
                || local.uncompressed_size_hint() != header.uncompressed_size_hint())
        {
            return Err(ToolError::InvalidManifest);
        }
        if header.flags().has_data_descriptor() {
            let descriptor = entry
                .reader()
                .data_descriptor()
                .map_err(|_| ToolError::InvalidManifest)?
                .ok_or(ToolError::InvalidManifest)?;
            if descriptor.crc32() != header.crc32()
                || descriptor.compressed_size() != header.compressed_size_hint()
                || descriptor.uncompressed_size() != header.uncompressed_size_hint()
            {
                return Err(ToolError::InvalidManifest);
            }
        }
        let (start, end) = entry.compressed_data_range();
        let header_start = header.local_header_offset();
        if header_start >= start
            || end < start
            || end > archive.directory_offset()
            || end > package.archive_bytes
            || end - start != header.compressed_size_hint()
            || ranges.iter().any(|(s, e)| header_start < *e && *s < end)
        {
            return Err(ToolError::InvalidManifest);
        }
        ranges.push((header_start, end));
        names.push(name.to_owned());
        if component.is_some_and(|c| c.name == component_name) {
            target = Some((wayfinder, method));
        }
    }
    if names.len() as u64 != archive.entries_hint()
        || (mapped.is_none() && names.len() != package.archive_components().count())
        || mapped.is_some_and(|(_, mappings)| {
            mappings
                .iter()
                .any(|m| !names.iter().any(|n| n == &m.member))
        })
    {
        return Err(ToolError::InvalidManifest);
    }
    let (wayfinder, method) = target.ok_or(ToolError::InvalidManifest)?;
    let entry = archive
        .get_entry(wayfinder)
        .map_err(|_| ToolError::InvalidManifest)?;
    let compressed = entry.reader();
    let mut decoded = if method == CompressionMethod::STORE {
        Decoded::Stored(compressed)
    } else {
        Decoded::Deflate(flate2::read::DeflateDecoder::new(compressed))
    };
    let mut verified = entry.verifying_reader(&mut decoded);
    let mut hash = Sha256::new();
    let mut written = 0u64;
    loop {
        check(cancel, deadline)?;
        let n = verified
            .read(&mut local_buffer)
            .map_err(|_| ToolError::InvalidManifest)?;
        if n == 0 {
            break;
        }
        written = written
            .checked_add(n as u64)
            .ok_or(ToolError::DownloadLimit)?;
        if written > selected.bytes || written > selected_limit {
            return Err(ToolError::UnexpectedSize);
        }
        check(cancel, deadline)?;
        output.write_all(&local_buffer[..n])?;
        hash.update(&local_buffer[..n]);
    }
    let _ = verified.into_inner();
    let compressed_size = entry.compressed_data_range().1 - entry.compressed_data_range().0;
    let consumed = match decoded {
        Decoded::Stored(_) => written,
        Decoded::Deflate(decoder) => decoder.total_in(),
    };
    if consumed != compressed_size {
        return Err(ToolError::InvalidManifest);
    }
    check(cancel, deadline)?;
    if written != selected.bytes {
        return Err(ToolError::UnexpectedSize);
    }
    if format!("{:x}", hash.finalize()) != selected.sha256.to_ascii_lowercase() {
        return Err(ToolError::HashMismatch);
    }
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{ArchiveComponent, ArchiveComponentRole, Ownership};
    use std::time::Duration;
    fn zip(names: &[&str], data: &[u8], deflate: bool) -> Vec<u8> {
        let mut bytes = Vec::new();
        let mut archive = rawzip::ZipArchiveWriter::new(&mut bytes);
        for name in names {
            if name.ends_with('/') {
                archive.new_dir(*name).create().unwrap();
                continue;
            }
            let (mut entry, config) = archive
                .new_file(*name)
                .compression_method(if deflate {
                    CompressionMethod::DEFLATE
                } else {
                    CompressionMethod::STORE
                })
                .start()
                .unwrap();
            if deflate {
                let encoder =
                    flate2::write::DeflateEncoder::new(&mut entry, flate2::Compression::default());
                let mut writer = config.wrap(encoder);
                writer.write_all(data).unwrap();
                let (encoder, descriptor) = writer.finish().unwrap();
                encoder.finish().unwrap();
                entry.finish(descriptor).unwrap();
            } else {
                let mut writer = config.wrap(&mut entry);
                writer.write_all(data).unwrap();
                let (_, descriptor) = writer.finish().unwrap();
                entry.finish(descriptor).unwrap();
            }
        }
        archive.finish().unwrap();
        bytes
    }
    fn package(bytes: &[u8], data: &[u8]) -> ArchivePackage {
        ArchivePackage {
            notices: vec![],
            version: 1,
            layout: None,
            tool: "deno".into(),
            tool_version: "fixture-1".into(),
            target: "linux-x64".into(),
            source_url: "https://example.invalid/runtime.zip".into(),
            license: "MIT".into(),
            ownership: Ownership::Managed,
            archive_bytes: bytes.len() as u64,
            archive_sha256: format!("{:x}", Sha256::digest(bytes)),
            components: vec![ArchiveComponent {
                name: "deno".into(),
                bytes: data.len() as u64,
                sha256: format!("{:x}", Sha256::digest(data)),
                role: ArchiveComponentRole::Executable,
            }],
        }
    }
    fn run(bytes: &[u8], p: &ArchivePackage) -> Result<Vec<u8>, ToolError> {
        let mut source = tempfile::tempfile().unwrap();
        source.write_all(bytes).unwrap();
        let mut output = tempfile::tempfile().unwrap();
        extract_verified_zip(
            &source,
            p,
            "deno",
            &mut output,
            &AtomicBool::new(false),
            Instant::now() + Duration::from_secs(5),
        )?;
        output.seek(SeekFrom::Start(0)).unwrap();
        let mut actual = Vec::new();
        output.read_to_end(&mut actual).unwrap();
        source.seek(SeekFrom::Start(0)).unwrap();
        let mut retained = Vec::new();
        source.read_to_end(&mut retained).unwrap();
        assert_eq!(retained, bytes);
        Ok(actual)
    }
    fn v3_external_notice(bytes: &[u8], data: &[u8]) -> ArchivePackage {
        let mut p = package(bytes, data);
        p.version = 3;
        p.components.push(ArchiveComponent {
            name: "LICENSE".into(),
            bytes: 100,
            sha256: "c".repeat(64),
            role: ArchiveComponentRole::Notice,
        });
        p.notices.push(crate::tools::PinnedArchiveNotice {
            component: "LICENSE".into(),
            source_url: "https://example.invalid/revision/LICENSE".into(),
            revision: "d".repeat(40),
        });
        p
    }
    #[test]
    fn v3_external_notice_is_not_an_archive_member_or_selection() {
        for deflate in [false, true] {
            let data = b"synthetic archive executable";
            let bytes = zip(&["deno"], data, deflate);
            let p = v3_external_notice(&bytes, data);
            assert!(p.validate().is_ok());
            assert_eq!(run(&bytes, &p).unwrap(), data);
            let mut source = tempfile::tempfile().unwrap();
            source.write_all(&bytes).unwrap();
            let mut output = tempfile::tempfile().unwrap();
            let error = extract_verified_zip(
                &source,
                &p,
                "LICENSE",
                &mut output,
                &AtomicBool::new(false),
                Instant::now() + Duration::from_secs(5),
            )
            .unwrap_err();
            assert!(matches!(error, ToolError::InvalidManifest));
            assert_eq!(output.metadata().unwrap().len(), 0);
            assert_eq!(output.stream_position().unwrap(), 0);
            source.seek(SeekFrom::Start(0)).unwrap();
            let mut retained = Vec::new();
            source.read_to_end(&mut retained).unwrap();
            assert_eq!(retained, bytes);
        }
    }
    #[test]
    fn v3_external_notice_name_inside_archive_rejects_supply_confusion() {
        for names in [vec!["deno", "LICENSE"], vec!["LICENSE", "deno"]] {
            let data = b"synthetic archive executable";
            let bytes = zip(&names, data, false);
            let p = v3_external_notice(&bytes, data);
            assert!(p.validate().is_ok());
            let mut source = tempfile::tempfile().unwrap();
            source.write_all(&bytes).unwrap();
            let mut output = tempfile::tempfile().unwrap();
            let error = extract_verified_zip(
                &source,
                &p,
                "deno",
                &mut output,
                &AtomicBool::new(false),
                Instant::now() + Duration::from_secs(5),
            )
            .unwrap_err();
            assert!(matches!(error, ToolError::InvalidManifest));
            assert_eq!(output.metadata().unwrap().len(), 0);
            source.seek(SeekFrom::Start(0)).unwrap();
            let mut retained = Vec::new();
            source.read_to_end(&mut retained).unwrap();
            assert_eq!(retained, bytes);
        }
    }
    #[test]
    fn stored_and_deflate_extract_exact_bytes() {
        for deflate in [false, true] {
            let data = b"synthetic runtime bytes";
            let z = zip(&["deno"], data, deflate);
            assert_eq!(run(&z, &package(&z, data)).unwrap(), data);
        }
    }
    #[test]
    fn untrusted_hash_and_component_hash_rejected() {
        let data = b"synthetic";
        let z = zip(&["deno"], data, false);
        let mut p = package(&z, data);
        p.archive_sha256 = "a".repeat(64);
        assert!(matches!(run(&z, &p), Err(ToolError::HashMismatch)));
        p = package(&z, data);
        p.components[0].sha256 = "b".repeat(64);
        assert!(matches!(run(&z, &p), Err(ToolError::HashMismatch)));
    }
    #[test]
    fn unsafe_names_duplicate_and_entry_flood_rejected() {
        let good = zip(&["deno"], b"data", false);
        let central = good.windows(4).position(|w| w == b"PK\x01\x02").unwrap();
        // Writer normalizes paths; mutate actual local and central names instead.
        for name in [b"../x", b"/eno", b"a\\xx", b"C:xx"] {
            let mut z = good.clone();
            z[30..34].copy_from_slice(name);
            z[central + 46..central + 50].copy_from_slice(name);
            assert!(run(&z, &package(&z, b"data")).is_err());
        }
        for names in [vec!["deno", "deno"], vec!["deno"; 33]] {
            let z = zip(&names, b"data", false);
            assert!(run(&z, &package(&z, b"data")).is_err());
        }
    }
    #[test]
    fn local_central_crc_symlink_and_encryption_rejected() {
        let good = zip(&["deno"], b"data", false);
        let central = good.windows(4).position(|w| w == b"PK\x01\x02").unwrap();
        for mutation in 0..5 {
            let mut z = good.clone();
            match mutation {
                0 => z[30] = b'x',         // local name disagrees with central.
                1 => z[central + 16] ^= 1, // central CRC mismatch actual bytes.
                2 => {
                    z[central + 5] = 3;
                    z[central + 38..central + 42]
                        .copy_from_slice(&(0o120777u32 << 16).to_le_bytes());
                }
                3 => {
                    z[6] |= 1;
                    z[central + 8] |= 1;
                }
                _ => {
                    z[8..10].copy_from_slice(&99u16.to_le_bytes());
                    z[central + 10..central + 12].copy_from_slice(&99u16.to_le_bytes());
                }
            }
            assert!(
                run(&z, &package(&z, b"data")).is_err(),
                "mutation {mutation}"
            );
        }
    }
    #[test]
    fn data_descriptor_corruption_and_archive_size_rejected() {
        let good = zip(&["deno"], b"data", true);
        let descriptor = good.windows(4).position(|w| w == b"PK\x07\x08").unwrap();
        for offset in [4, 8, 12] {
            let mut z = good.clone();
            z[descriptor + offset] ^= 1;
            assert!(run(&z, &package(&z, b"data")).is_err());
        }
        let mut p = package(&good, b"data");
        p.archive_bytes += 1;
        assert!(matches!(run(&good, &p), Err(ToolError::UnexpectedSize)));
    }
    #[test]
    fn trailing_compressed_garbage_and_count_hint_mismatch_rejected() {
        let good = zip(&["deno"], b"data", true);
        let descriptor = good.windows(4).position(|w| w == b"PK\x07\x08").unwrap();
        let central = good.windows(4).position(|w| w == b"PK\x01\x02").unwrap();
        let eocd = good.windows(4).position(|w| w == b"PK\x05\x06").unwrap();
        let mut z = good.clone();
        z.splice(descriptor..descriptor, [1, 2, 3]);
        for at in [descriptor + 3 + 8, central + 3 + 20] {
            let value = u32::from_le_bytes(z[at..at + 4].try_into().unwrap()) + 3;
            z[at..at + 4].copy_from_slice(&value.to_le_bytes());
        }
        z[eocd + 3 + 16..eocd + 3 + 20].copy_from_slice(&((central + 3) as u32).to_le_bytes());
        assert!(run(&z, &package(&z, b"data")).is_err());
        let mut z = good;
        z[eocd + 8..eocd + 10].copy_from_slice(&2u16.to_le_bytes());
        z[eocd + 10..eocd + 12].copy_from_slice(&2u16.to_le_bytes());
        assert!(run(&z, &package(&z, b"data")).is_err());
    }
    #[test]
    fn expansion_lie_cancel_deadline_and_nonempty_output_rejected() {
        let data = vec![0u8; 1024 * 1024];
        let z = zip(&["deno"], &data, true);
        let mut p = package(&z, &data);
        p.components[0].bytes = 1;
        assert!(matches!(run(&z, &p), Err(ToolError::UnexpectedSize)));
        let p = package(&z, &data);
        let mut source = tempfile::tempfile().unwrap();
        source.write_all(&z).unwrap();
        let mut output = tempfile::tempfile().unwrap();
        assert!(matches!(
            extract_verified_zip(
                &source,
                &p,
                "deno",
                &mut output,
                &AtomicBool::new(true),
                Instant::now() + Duration::from_secs(1)
            ),
            Err(ToolError::Cancelled)
        ));
        assert!(matches!(
            extract_verified_zip(
                &source,
                &p,
                "deno",
                &mut output,
                &AtomicBool::new(false),
                Instant::now()
            ),
            Err(ToolError::TimedOut)
        ));
        output.write_all(b"retained").unwrap();
        output.seek(SeekFrom::Start(0)).unwrap();
        assert!(matches!(
            extract_verified_zip(
                &source,
                &p,
                "deno",
                &mut output,
                &AtomicBool::new(false),
                Instant::now() + Duration::from_secs(1)
            ),
            Err(ToolError::UnsafePath)
        ));
    }
    fn mapped_package(z: &[u8], data: &[u8]) -> ArchivePackage {
        let mut p = package(z, data);
        p.version = 2;
        p.layout = Some(ArchiveLayout::Zip {
            root: "supply".into(),
            mappings: vec![super::super::ArchiveMemberMapping {
                member: "supply/bin/deno".into(),
                component: "deno".into(),
            }],
        });
        p
    }
    #[test]
    fn mapped_nested_stored_and_deflate_with_unselected_members() {
        for deflate in [false, true] {
            let data = b"selected bytes";
            let z = zip(
                &[
                    "supply/",
                    "supply/bin/",
                    "supply/bin/deno",
                    "supply/docs/license",
                    "supply/extra",
                ],
                data,
                deflate,
            );
            assert_eq!(run(&z, &mapped_package(&z, data)).unwrap(), data);
            let mut p = mapped_package(&z, data);
            p.components[0].sha256 = "a".repeat(64);
            assert!(matches!(run(&z, &p), Err(ToolError::HashMismatch)));
        }
    }
    #[test]
    fn mapped_missing_duplicate_outside_and_file_ancestor_rejected() {
        for names in [
            vec!["supply/docs/license"],
            vec!["supply/bin/deno", "supply/BIN/DENO"],
            vec!["supply/bin/deno", "outside/license"],
            vec!["supply/bin", "supply/bin/deno"],
            vec!["supply/bin/deno", "supply/bin"],
        ] {
            let z = zip(&names, b"data", false);
            assert!(run(&z, &mapped_package(&z, b"data")).is_err());
        }
    }
    #[test]
    fn mapped_casefold_file_ancestor_both_orders_rejected() {
        for names in [
            vec!["supply/bin/deno", "supply/FOO", "supply/foo/bar"],
            vec!["supply/bin/deno", "supply/foo/bar", "supply/FOO"],
        ] {
            let z = zip(&names, b"data", false);
            assert!(matches!(
                run(&z, &mapped_package(&z, b"data")),
                Err(ToolError::UnsafePath)
            ));
        }
    }
    #[test]
    fn mapped_unselected_headers_enforce_size_type_and_local_consistency() {
        let good = zip(&["supply/bin/deno", "supply/docs/license"], b"data", false);
        let central: Vec<_> = good
            .windows(4)
            .enumerate()
            .filter_map(|(i, w)| (w == b"PK\x01\x02").then_some(i))
            .collect();
        for mutation in 0..3 {
            let mut z = good.clone();
            let at = central[1];
            match mutation {
                0 => {
                    z[at + 24..at + 28].copy_from_slice(&((MAPPED_TOTAL + 1) as u32).to_le_bytes())
                }
                1 => {
                    z[at + 5] = 3;
                    z[at + 38..at + 42].copy_from_slice(&(0o120777u32 << 16).to_le_bytes());
                }
                _ => z[at + 46] = b'x',
            }
            assert!(run(&z, &mapped_package(&z, b"data")).is_err());
        }
    }
}
