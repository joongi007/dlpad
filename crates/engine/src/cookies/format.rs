//! OS-independent Netscape format policy and volatile secret-buffer cleanup.
use super::{CookieImportError, MAX_COOKIE_BYTES, MAX_COOKIE_ENTRIES, MAX_COOKIE_LINE_BYTES};
use std::{fmt, sync::atomic::Ordering};

pub(super) struct SensitiveBytes(pub(super) Vec<u8>);
impl fmt::Debug for SensitiveBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted>")
    }
}
fn scrub(bytes: &mut [u8]) {
    for byte in bytes {
        // SAFETY: each pointer comes from an exclusive live byte reference.
        unsafe { std::ptr::write_volatile(byte, 0) };
    }
    std::sync::atomic::compiler_fence(Ordering::SeqCst);
}
impl Drop for SensitiveBytes {
    fn drop(&mut self) {
        scrub(&mut self.0);
    }
}
pub(super) struct SensitiveBlock(pub(super) [u8; 4096]);
impl Drop for SensitiveBlock {
    fn drop(&mut self) {
        scrub(&mut self.0);
    }
}

pub(super) fn validate(data: &[u8]) -> Result<usize, CookieImportError> {
    if data.len() > MAX_COOKIE_BYTES {
        return Err(CookieImportError::TooLarge);
    }
    if data.contains(&0) {
        return Err(CookieImportError::InvalidFormat);
    }
    let text = std::str::from_utf8(data).map_err(|_| CookieImportError::InvalidFormat)?;
    let mut count = 0;
    for (index, raw) in text.split('\n').enumerate() {
        if raw.len() > MAX_COOKIE_LINE_BYTES {
            return Err(CookieImportError::LineTooLong);
        }
        let line = raw.strip_suffix('\r').unwrap_or(raw);
        if index == 0 {
            if line != "# Netscape HTTP Cookie File" && line != "# HTTP Cookie File" {
                return Err(CookieImportError::InvalidFormat);
            }
            continue;
        }
        if line.is_empty() {
            continue;
        }
        let line = if let Some(cookie) = line.strip_prefix("#HttpOnly_") {
            cookie
        } else if line.starts_with('#') {
            continue;
        } else {
            line
        };
        let mut columns = line.split('\t');
        let mut fields = [""; 7];
        for field in &mut fields {
            *field = columns.next().ok_or(CookieImportError::InvalidFormat)?;
        }
        if columns.next().is_some()
            || fields
                .iter()
                .any(|field| field.chars().any(char::is_control))
        {
            return Err(CookieImportError::InvalidFormat);
        }
        let domain = fields[0];
        if domain.is_empty()
            || domain == "."
            || domain.chars().any(char::is_whitespace)
            || !matches!(fields[1], "TRUE" | "FALSE")
            || (fields[1] == "TRUE") != domain.starts_with('.')
            || !fields[2].starts_with('/')
            || !matches!(fields[3], "TRUE" | "FALSE")
            || (!fields[4].is_empty() && fields[4].parse::<i64>().is_err())
        {
            return Err(CookieImportError::InvalidFormat);
        }
        count += 1;
        if count > MAX_COOKIE_ENTRIES {
            return Err(CookieImportError::TooManyEntries);
        }
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    const HEADER: &str = "# Netscape HTTP Cookie File\n";
    const COOKIE: &str = ".fixture.invalid\tTRUE\t/\tTRUE\t0\tSESSION\tsynthetic-secret\n";
    fn valid() -> Vec<u8> {
        format!("{HEADER}{COOKIE}").into_bytes()
    }
    #[test]
    fn literal_lf_crlf_httponly_and_zero_entries_are_unchanged() {
        for bytes in [
            valid(),
            format!("{HEADER}#HttpOnly_{COOKIE}").into_bytes(),
            format!("{HEADER}{COOKIE}")
                .replace('\n', "\r\n")
                .into_bytes(),
        ] {
            let original = bytes.clone();
            assert_eq!(validate(&bytes), Ok(1));
            assert_eq!(bytes, original);
        }
        for text in [
            HEADER,
            "# HTTP Cookie File",
            "# HTTP Cookie File\r\n# comment\r\n",
        ] {
            assert_eq!(validate(text.as_bytes()), Ok(0));
        }
        assert_eq!(
            validate(format!("{HEADER}.fixture.invalid\tTRUE\t/\tFALSE\t-1\t\t\n").as_bytes()),
            Ok(1)
        );
        assert_eq!(
            validate(format!("{HEADER}fixture.invalid\tFALSE\t/\tTRUE\t\tN\t\n").as_bytes()),
            Ok(1)
        );
    }
    #[test]
    fn malformed_fields_and_headers_are_rejected() {
        for line in [
            "x",
            ".fixture.invalid\tTRUE\t/\tTRUE\t0\tN",
            ".fixture.invalid\tTRUE\t/\tTRUE\t0\tN\tV\textra",
            ".fixture.invalid\tFALSE\t/\tTRUE\t0\tN\tV",
            "fixture.invalid\tTRUE\t/\tTRUE\t0\tN\tV",
            ".fixture.invalid\tTRUE\trelative\tTRUE\t0\tN\tV",
            ".fixture.invalid\tTRUE\t/\tMAYBE\t0\tN\tV",
            ".fixture.invalid\tTRUE\t/\tTRUE\t9223372036854775808\tN\tV",
            ".fixture.invalid\tTRUE\t/\tTRUE\t0\tN\tV\u{1f}",
            ".fixture invalid\tTRUE\t/\tTRUE\t0\tN\tV",
            "#HttpOnly_bad",
        ] {
            assert_eq!(
                validate(format!("{HEADER}{line}\n").as_bytes()),
                Err(CookieImportError::InvalidFormat)
            );
        }
        for bytes in [
            b"".as_slice(),
            b"bad header\n",
            b"# Netscape HTTP Cookie File\n\0",
            b"# HTTP Cookie File\n\xff",
        ] {
            assert_eq!(validate(bytes), Err(CookieImportError::InvalidFormat));
        }
    }
    fn padded(total: usize) -> Vec<u8> {
        let mut bytes = valid();
        while bytes.len() < total {
            let remaining = total - bytes.len();
            if remaining == 1 {
                bytes.push(b'\n');
                break;
            }
            let line = (remaining - 1).min(MAX_COOKIE_LINE_BYTES);
            bytes.push(b'#');
            bytes.resize(bytes.len() + line - 1, b'x');
            bytes.push(b'\n');
        }
        bytes
    }
    #[test]
    fn exact_total_byte_bound_and_error_precedence() {
        let mut bytes = padded(MAX_COOKIE_BYTES);
        assert_eq!(validate(&bytes), Ok(1));
        bytes.push(0);
        assert_eq!(validate(&bytes), Err(CookieImportError::TooLarge));
        let mut bytes = vec![b'x'; MAX_COOKIE_LINE_BYTES + 1];
        bytes[0] = 0;
        assert_eq!(validate(&bytes), Err(CookieImportError::InvalidFormat));
        bytes[0] = 0xff;
        assert_eq!(validate(&bytes), Err(CookieImportError::InvalidFormat));
    }
    #[test]
    fn physical_line_limit_includes_cr_and_excludes_lf() {
        let mut bytes = HEADER.as_bytes().to_vec();
        bytes.push(b'#');
        bytes.extend(std::iter::repeat_n(b'x', MAX_COOKIE_LINE_BYTES - 2));
        bytes.extend_from_slice(b"\r\n");
        assert_eq!(validate(&bytes), Ok(0));
        bytes.insert(bytes.len() - 2, b'x');
        assert_eq!(validate(&bytes), Err(CookieImportError::LineTooLong));
    }
    #[test]
    fn exact_entry_bound_counts_httponly_but_not_comments() {
        let mut bytes = HEADER.as_bytes().to_vec();
        for _ in 0..MAX_COOKIE_ENTRIES {
            bytes.extend_from_slice(b"#HttpOnly_");
            bytes.extend_from_slice(COOKIE.as_bytes());
        }
        bytes.extend_from_slice(b"# comment\n\n");
        assert_eq!(validate(&bytes), Ok(MAX_COOKIE_ENTRIES));
        bytes.extend_from_slice(COOKIE.as_bytes());
        assert_eq!(validate(&bytes), Err(CookieImportError::TooManyEntries));
    }
    #[test]
    fn secret_buffer_debug_and_scrub_are_os_independent() {
        let bytes = SensitiveBytes(b"synthetic-secret".to_vec());
        assert_eq!(format!("{bytes:?}"), "<redacted>");
        let mut block = SensitiveBlock([0x42; 4096]);
        scrub(&mut block.0);
        assert!(block.0.iter().all(|b| *b == 0));
        let mut bytes = bytes;
        scrub(&mut bytes.0);
        assert!(bytes.0.iter().all(|b| *b == 0));
    }
}
