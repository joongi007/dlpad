//! Read-only validation of the current preview installer ZIP profile.
//! An observation is not extracted content, installation authority or executable trust.
use super::{MAX_PACKAGE_BYTES, StagedPackage};
use crate::app_install::manifest::BundleManifest;
use rawzip::{CompressionMethod, ReaderAt, ZipArchive, ZipArchiveEntryWayfinder};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    io::{self, Read, Seek, SeekFrom},
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};
const BUFFER: usize = 64 * 1024;
const MAX_DOC: u64 = 16 * 1024 * 1024;
const MAX_EXE: u64 = 128 * 1024 * 1024;
const MAX_SMALL: u64 = 64 * 1024;
const TARGET: &str = "x86_64-pc-windows-msvc";
const EXPECTED: [&str; 7] = [
    "dlpad-install.exe",
    "payload/manifest.json",
    "payload/dlpad.exe",
    "payload/LICENSE",
    "payload/THIRD-PARTY-NOTICES.txt",
    "payload/README.txt",
    "payload/binding-record.txt",
];
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BundleError {
    Cancelled,
    TimedOut,
    Io(io::ErrorKind),
    Identity,
    OuterSize,
    OuterHash,
    Zip,
    Namespace,
    Limit,
    Manifest,
    MemberHash,
    Binding,
}
impl std::fmt::Display for BundleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "preview bundle validation failed ({self:?})")
    }
}
impl std::error::Error for BundleError {}
#[derive(Debug, Clone)]
pub struct MemberObservation {
    name: String,
    bytes: u64,
    sha256: [u8; 32],
}
impl MemberObservation {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn bytes(&self) -> u64 {
        self.bytes
    }
    pub fn sha256(&self) -> &[u8; 32] {
        &self.sha256
    }
}
/// Bounded content facts only. Retain and revalidate the original StagedPackage for further operations.
#[derive(Debug)]
pub struct BundleObservation {
    version: String,
    sequence: u64,
    package_bytes: u64,
    package_sha256: [u8; 32],
    members: Vec<MemberObservation>,
}
impl BundleObservation {
    pub fn version(&self) -> &str {
        &self.version
    }
    pub fn sequence(&self) -> u64 {
        self.sequence
    }
    pub fn package_bytes(&self) -> u64 {
        self.package_bytes
    }
    pub fn package_sha256(&self) -> &[u8; 32] {
        &self.package_sha256
    }
    pub fn members(&self) -> &[MemberObservation] {
        &self.members
    }
}
fn checkpoint(cancel: &AtomicBool, deadline: Instant) -> Result<(), BundleError> {
    if cancel.load(Ordering::Acquire) {
        return Err(BundleError::Cancelled);
    }
    if Instant::now() >= deadline {
        return Err(BundleError::TimedOut);
    }
    Ok(())
}
fn io(error: io::Error) -> BundleError {
    BundleError::Io(error.kind())
}
fn member_limit(name: &str) -> u64 {
    match name {
        "dlpad-install.exe" | "payload/dlpad.exe" => MAX_EXE,
        "payload/manifest.json" | "payload/binding-record.txt" => MAX_SMALL,
        _ => MAX_DOC,
    }
}
fn hash_outer(
    source: &mut (impl Read + Seek),
    bytes: u64,
    expected: &[u8; 32],
    cancel: &AtomicBool,
    deadline: Instant,
) -> Result<(), BundleError> {
    checkpoint(cancel, deadline)?;
    if bytes == 0
        || bytes > MAX_PACKAGE_BYTES
        || source.seek(SeekFrom::End(0)).map_err(io)? != bytes
    {
        return Err(BundleError::OuterSize);
    }
    source.seek(SeekFrom::Start(0)).map_err(io)?;
    let mut buffer = [0u8; BUFFER];
    let mut digest = Sha256::new();
    let mut count = 0u64;
    loop {
        checkpoint(cancel, deadline)?;
        let limit = usize::try_from(bytes - count + 1)
            .unwrap_or(BUFFER)
            .min(BUFFER);
        let n = source.read(&mut buffer[..limit]).map_err(io)?;
        checkpoint(cancel, deadline)?;
        if n == 0 {
            break;
        }
        count = count.checked_add(n as u64).ok_or(BundleError::OuterSize)?;
        if count > bytes {
            return Err(BundleError::OuterSize);
        }
        digest.update(&buffer[..n]);
    }
    if count != bytes {
        return Err(BundleError::OuterSize);
    }
    let actual: [u8; 32] = digest.finalize().into();
    if &actual != expected {
        return Err(BundleError::OuterHash);
    }
    source.seek(SeekFrom::Start(0)).map_err(io)?;
    Ok(())
}

/// Checks native capability and signed whole-package bytes before and after ZIP interpretation.
/// Does not reopen package.path(), create output files, install or execute any member.
pub fn inspect_staged_bundle(
    package: &mut StagedPackage,
    cancel: &AtomicBool,
    deadline: Instant,
) -> Result<BundleObservation, BundleError> {
    checkpoint(cancel, deadline)?;
    package
        .verify_current()
        .map_err(|_| BundleError::Identity)?;
    let version = package.version().to_owned();
    let sequence = package.sequence();
    let bytes = package.bytes();
    let digest = *package.sha256();
    let result = inspect_reader(
        package.reader_mut(),
        &version,
        sequence,
        bytes,
        &digest,
        cancel,
        deadline,
    );
    package
        .verify_current()
        .map_err(|_| BundleError::Identity)?;
    result
}

struct Member {
    name: String,
    wayfinder: ZipArchiveEntryWayfinder,
    method: CompressionMethod,
    bytes: u64,
}
enum Decoded<R: Read> {
    Stored(R),
    Deflate(flate2::read::DeflateDecoder<R>),
}
impl<R: Read> Read for Decoded<R> {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Stored(r) => r.read(out),
            Self::Deflate(r) => r.read(out),
        }
    }
}
fn read_member<R: ReaderAt>(
    archive: &ZipArchive<R>,
    member: &Member,
    retain: bool,
    cancel: &AtomicBool,
    deadline: Instant,
) -> Result<(MemberObservation, Vec<u8>), BundleError> {
    checkpoint(cancel, deadline)?;
    let entry = archive
        .get_entry(member.wayfinder)
        .map_err(|_| BundleError::Zip)?;
    let reader = entry.reader();
    let mut decoded = if member.method == CompressionMethod::STORE {
        Decoded::Stored(reader)
    } else {
        Decoded::Deflate(flate2::read::DeflateDecoder::new(reader))
    };
    let mut verified = entry.verifying_reader(&mut decoded);
    let mut digest = Sha256::new();
    let mut count = 0u64;
    let mut buffer = [0u8; BUFFER];
    let mut kept = Vec::new();
    loop {
        checkpoint(cancel, deadline)?;
        let limit = usize::try_from(member.bytes - count + 1)
            .unwrap_or(BUFFER)
            .min(BUFFER);
        let n = verified
            .read(&mut buffer[..limit])
            .map_err(|_| BundleError::Zip)?;
        checkpoint(cancel, deadline)?;
        if n == 0 {
            break;
        }
        count = count.checked_add(n as u64).ok_or(BundleError::Limit)?;
        if count > member.bytes || count > member_limit(&member.name) {
            return Err(BundleError::Limit);
        }
        digest.update(&buffer[..n]);
        if retain {
            kept.extend_from_slice(&buffer[..n]);
        }
    }
    let _ = verified.into_inner();
    let compressed_size = entry.compressed_data_range().1 - entry.compressed_data_range().0;
    let consumed = match decoded {
        Decoded::Stored(_) => count,
        Decoded::Deflate(decoder) => decoder.total_in(),
    };
    if consumed != compressed_size || count != member.bytes {
        return Err(BundleError::Zip);
    }
    Ok((
        MemberObservation {
            name: member.name.clone(),
            bytes: count,
            sha256: digest.finalize().into(),
        },
        kept,
    ))
}
fn u16_at(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes(bytes[at..at + 2].try_into().expect("fixed header field"))
}
fn u32_at(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(bytes[at..at + 4].try_into().expect("fixed header field"))
}

#[derive(Deserialize)]
struct Binding {
    schema: u32,
    version: String,
    target: String,
    development_only: bool,
    release_ready: bool,
    signed_distribution: bool,
    complete_legal_coverage: bool,
    binaries: Binaries,
    mapping: Mapping,
}
#[derive(Deserialize)]
struct Binaries {
    #[serde(rename = "dlpad-cpu.exe")]
    cpu: Pin,
    #[serde(rename = "dlpad-install.exe")]
    installer: Pin,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Mapping {
    #[serde(rename = "dlpad-cpu.exe")]
    cpu: String,
    #[serde(rename = "dlpad-install.exe")]
    installer: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Pin {
    bytes: u64,
    sha256: String,
}
fn pin_matches(pin: &Pin, member: &MemberObservation) -> bool {
    pin.bytes == member.bytes
        && pin.sha256
            == member
                .sha256
                .iter()
                .map(|v| format!("{v:02x}"))
                .collect::<String>()
}

fn inspect_reader(
    source: &mut (impl Read + Seek),
    version: &str,
    sequence: u64,
    bytes: u64,
    digest: &[u8; 32],
    cancel: &AtomicBool,
    deadline: Instant,
) -> Result<BundleObservation, BundleError> {
    hash_outer(source, bytes, digest, cancel, deadline)?;
    let members = parse_zip(source, version, cancel, deadline)?;
    hash_outer(source, bytes, digest, cancel, deadline)?;
    Ok(BundleObservation {
        version: version.to_owned(),
        sequence,
        package_bytes: bytes,
        package_sha256: *digest,
        members,
    })
}
fn parse_zip(
    source: &mut (impl Read + Seek),
    version: &str,
    cancel: &AtomicBool,
    deadline: Instant,
) -> Result<Vec<MemberObservation>, BundleError> {
    let mut buffer = [0u8; BUFFER];
    let mut local_buffer = [0u8; BUFFER];
    let length = source.seek(SeekFrom::End(0)).map_err(io)?;
    let archive = ZipArchive::from_seekable(source, &mut buffer).map_err(|_| BundleError::Zip)?;
    if archive.entries_hint() != EXPECTED.len() as u64
        || archive.end_offset() != length
        || archive.comment().remaining() != 0
    {
        return Err(BundleError::Zip);
    }
    let mut entries = archive.entries(&mut buffer);
    let mut members = Vec::new();
    let mut ranges = Vec::new();
    let mut expanded = 0u64;
    let mut central = archive.directory_offset();
    while let Some(header) = entries.next_entry().map_err(|_| BundleError::Zip)? {
        checkpoint(cancel, deadline)?;
        if members.len() == EXPECTED.len() {
            return Err(BundleError::Limit);
        }
        let path = header.file_path();
        let raw = path.as_ref();
        let name = std::str::from_utf8(raw).map_err(|_| BundleError::Namespace)?;
        if !EXPECTED.contains(&name)
            || members
                .iter()
                .any(|m: &Member| m.name.eq_ignore_ascii_case(name))
            || header.is_dir()
            || header.mode().value() & 0o170000 != 0o100000
        {
            return Err(BundleError::Namespace);
        }
        if !header.extra_fields().remaining_bytes().is_empty()
            || !header.comment().as_bytes().is_empty()
            || header.central_directory_offset() != central
        {
            return Err(BundleError::Zip);
        }
        central = central
            .checked_add(46 + raw.len() as u64)
            .ok_or(BundleError::Limit)?;
        let method = header.compression_method();
        let flags = header.flags();
        let allowed = if method == CompressionMethod::DEFLATE {
            0x080e
        } else {
            0x0808
        };
        if (method != CompressionMethod::STORE && method != CompressionMethod::DEFLATE)
            || flags.bits() & !allowed != 0
        {
            return Err(BundleError::Zip);
        }
        let size = header.uncompressed_size_hint();
        if size == 0 || size > member_limit(name) {
            return Err(BundleError::Limit);
        }
        expanded = expanded.checked_add(size).ok_or(BundleError::Limit)?;
        if expanded > MAX_PACKAGE_BYTES {
            return Err(BundleError::Limit);
        }
        let wayfinder = header.wayfinder();
        let entry = archive.get_entry(wayfinder).map_err(|_| BundleError::Zip)?;
        let local = entry
            .local_header(&mut local_buffer)
            .map_err(|_| BundleError::Zip)?;
        if local.file_path().as_ref() != raw
            || local.flags().bits() != flags.bits()
            || local.compression_method() != method
            || !local.extra_fields().remaining_bytes().is_empty()
        {
            return Err(BundleError::Zip);
        }
        if !flags.has_data_descriptor()
            && (local.crc32() != header.crc32()
                || local.compressed_size_hint() != header.compressed_size_hint()
                || local.uncompressed_size_hint() != size)
        {
            return Err(BundleError::Zip);
        }
        let (start, end) = entry.compressed_data_range();
        let offset = header.local_header_offset();
        if start
            != offset
                .checked_add(30 + raw.len() as u64)
                .ok_or(BundleError::Limit)?
            || end < start
            || end - start != header.compressed_size_hint()
        {
            return Err(BundleError::Zip);
        }
        let mut physical_end = end;
        if flags.has_data_descriptor() {
            let descriptor = entry
                .reader()
                .data_descriptor()
                .map_err(|_| BundleError::Zip)?
                .ok_or(BundleError::Zip)?;
            if descriptor.crc32() != header.crc32()
                || descriptor.compressed_size() != header.compressed_size_hint()
                || descriptor.uncompressed_size() != size
            {
                return Err(BundleError::Zip);
            }
            let mut raw_descriptor = [0u8; 16];
            archive
                .get_ref()
                .read_exact_at(&mut raw_descriptor[..4], end)
                .map_err(io)?;
            let prefix = if u32_at(&raw_descriptor, 0) == 0x08074b50 {
                4
            } else {
                0
            };
            archive
                .get_ref()
                .read_exact_at(&mut raw_descriptor[..12 + prefix], end)
                .map_err(io)?;
            if u32_at(&raw_descriptor, prefix) != header.crc32()
                || u64::from(u32_at(&raw_descriptor, prefix + 4)) != header.compressed_size_hint()
                || u64::from(u32_at(&raw_descriptor, prefix + 8)) != size
            {
                return Err(BundleError::Zip);
            }
            physical_end = end
                .checked_add((12 + prefix) as u64)
                .ok_or(BundleError::Limit)?;
        }
        if offset >= start
            || physical_end > archive.directory_offset()
            || ranges.iter().any(|(s, e)| offset < *e && *s < physical_end)
        {
            return Err(BundleError::Zip);
        }
        ranges.push((offset, physical_end));
        members.push(Member {
            name: name.to_owned(),
            wayfinder,
            method,
            bytes: size,
        });
    }
    if members.len() != EXPECTED.len() || central != archive.eocd_offset() {
        return Err(BundleError::Zip);
    }
    ranges.sort_unstable();
    let mut physical = 0u64;
    for (start, end) in ranges {
        if start != physical {
            return Err(BundleError::Zip);
        }
        physical = end;
    }
    if physical != archive.directory_offset() {
        return Err(BundleError::Zip);
    }
    let mut eocd = [0u8; 22];
    archive
        .get_ref()
        .read_exact_at(&mut eocd, archive.eocd_offset())
        .map_err(io)?;
    if u32_at(&eocd, 0) != 0x06054b50
        || u16_at(&eocd, 4) != 0
        || u16_at(&eocd, 6) != 0
        || u16_at(&eocd, 8) != 7
        || u16_at(&eocd, 10) != 7
        || u16_at(&eocd, 20) != 0
        || u64::from(u32_at(&eocd, 12)) != central - archive.directory_offset()
        || u64::from(u32_at(&eocd, 16)) != archive.directory_offset()
    {
        return Err(BundleError::Zip);
    }
    let manifest_member = members
        .iter()
        .find(|m| m.name == "payload/manifest.json")
        .ok_or(BundleError::Manifest)?;
    let (_, manifest_raw) = read_member(&archive, manifest_member, true, cancel, deadline)?;
    let manifest =
        BundleManifest::read(manifest_raw.as_slice()).map_err(|_| BundleError::Manifest)?;
    if manifest.version != version
        || manifest.target != TARGET
        || !manifest.development_only
        || manifest.entries.len() != 5
    {
        return Err(BundleError::Manifest);
    }
    let required = [
        "dlpad.exe",
        "LICENSE",
        "THIRD-PARTY-NOTICES.txt",
        "README.txt",
        "binding-record.txt",
    ];
    if required
        .iter()
        .any(|name| !manifest.entries.iter().any(|e| e.name == *name))
    {
        return Err(BundleError::Manifest);
    }
    let mut observed = Vec::new();
    let mut binding_raw = Vec::new();
    for member in &members {
        let (item, retained) = read_member(
            &archive,
            member,
            member.name == "payload/binding-record.txt",
            cancel,
            deadline,
        )?;
        if let Some(leaf) = member.name.strip_prefix("payload/")
            && leaf != "manifest.json"
        {
            let expected = manifest
                .entries
                .iter()
                .find(|e| e.name == leaf)
                .ok_or(BundleError::Manifest)?;
            if expected.bytes != item.bytes
                || expected.sha256
                    != item
                        .sha256
                        .iter()
                        .map(|v| format!("{v:02x}"))
                        .collect::<String>()
            {
                return Err(BundleError::MemberHash);
            }
        }
        if member.name == "payload/binding-record.txt" {
            binding_raw = retained;
        }
        observed.push(item);
    }
    let binding: Binding =
        serde_json::from_slice(&binding_raw).map_err(|_| BundleError::Binding)?;
    let cpu = observed
        .iter()
        .find(|m| m.name == "payload/dlpad.exe")
        .ok_or(BundleError::Binding)?;
    let installer = observed
        .iter()
        .find(|m| m.name == "dlpad-install.exe")
        .ok_or(BundleError::Binding)?;
    if binding.schema != 1
        || binding.version != version
        || binding.target != TARGET
        || !binding.development_only
        || binding.release_ready
        || binding.signed_distribution
        || binding.complete_legal_coverage
        || binding.mapping.cpu != "payload/dlpad.exe"
        || binding.mapping.installer != "dlpad-install.exe"
        || !pin_matches(&binding.binaries.cpu, cpu)
        || !pin_matches(&binding.binaries.installer, installer)
    {
        return Err(BundleError::Binding);
    }
    checkpoint(cancel, deadline)?;
    Ok(observed)
}

#[cfg(test)]
mod tests;
