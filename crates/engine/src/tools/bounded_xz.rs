//! Original File digest gate and bounded single-stream XZ decoding.
use super::{TarExtractionRequest, ToolError, extract_tar_component};
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom},
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};
pub const MAX_XZ_MEMORY_KIB: u32 = 64 * 1024;
const MAX_SOURCE_BYTES: u64 = 256 * 1024 * 1024;
pub struct TarXzSourceSpec<'a> {
    pub bytes: u64,
    pub sha256: &'a str,
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
fn verify(
    source: &File,
    spec: &TarXzSourceSpec<'_>,
    cancel: &AtomicBool,
    deadline: Instant,
) -> Result<(), ToolError> {
    check(cancel, deadline)?;
    let metadata = source.metadata()?;
    if !metadata.is_file() || metadata.len() != spec.bytes {
        return Err(ToolError::UnexpectedSize);
    }
    let mut cursor = source;
    cursor.seek(SeekFrom::Start(0))?;
    let mut hash = Sha256::new();
    let mut bytes = 0u64;
    let mut buffer = [0u8; 65536];
    loop {
        check(cancel, deadline)?;
        let n = cursor.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        bytes = bytes
            .checked_add(n as u64)
            .ok_or(ToolError::DownloadLimit)?;
        if bytes > spec.bytes {
            return Err(ToolError::UnexpectedSize);
        }
        hash.update(&buffer[..n]);
    }
    if bytes != spec.bytes {
        return Err(ToolError::UnexpectedSize);
    }
    if format!("{:x}", hash.finalize()) != spec.sha256 {
        return Err(ToolError::HashMismatch);
    }
    Ok(())
}
struct SourceReader<'a> {
    file: &'a File,
    used: u64,
    limit: u64,
    cancel: &'a AtomicBool,
    deadline: Instant,
}
impl Read for SourceReader<'_> {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if self.cancel.load(Ordering::Acquire) {
            return Err(io::Error::other("owned XZ read cancelled"));
        }
        if Instant::now() >= self.deadline {
            return Err(io::ErrorKind::TimedOut.into());
        }
        let n = self.file.read(out)?;
        self.used = self
            .used
            .checked_add(n as u64)
            .ok_or(io::ErrorKind::InvalidData)?;
        if self.used > self.limit {
            return Err(io::ErrorKind::InvalidData.into());
        }
        Ok(n)
    }
}
/// Caller owns source namespace pins, output publication and partial-output cleanup.
/// The memory setting limits per-block LZMA2 decoder memory, not total process RSS.
pub fn extract_verified_tar_xz(
    source: &File,
    spec: TarXzSourceSpec<'_>,
    request: TarExtractionRequest<'_>,
    output: &mut File,
    cancel: &AtomicBool,
    deadline: Instant,
) -> Result<u64, ToolError> {
    check(cancel, deadline)?;
    if spec.bytes == 0
        || spec.bytes > MAX_SOURCE_BYTES
        || spec.sha256.len() != 64
        || !spec
            .sha256
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(ToolError::InvalidManifest);
    }
    if !output.metadata()?.is_file()
        || output.metadata()?.len() != 0
        || output.stream_position()? != 0
    {
        return Err(ToolError::UnsafePath);
    }
    verify(source, &spec, cancel, deadline)?;
    let mut cursor = source;
    cursor.seek(SeekFrom::Start(0))?;
    let reader = SourceReader {
        file: source,
        used: 0,
        limit: spec.bytes,
        cancel,
        deadline,
    };
    let mut decoder = lzma_rust2::XzReader::new_mem_limit(reader, false, MAX_XZ_MEMORY_KIB);
    let result = extract_tar_component(&mut decoder, request, output, cancel, deadline);
    check(cancel, deadline)?;
    let count = result?;
    if decoder.inner().used != spec.bytes {
        return Err(ToolError::UnexpectedSize);
    }
    drop(decoder);
    verify(source, &spec, cancel, deadline)?;
    check(cancel, deadline)?;
    Ok(count)
}
