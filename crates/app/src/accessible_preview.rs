//! Bounded UTF-8 views for native text layout; complete source bytes remain unchanged.
use dlpad_core::diagnostics::MAX_REPORT_BYTES;
const PREVIEW_PAGE_BYTES: usize = 8192;
// A UTF-8 boundary can retreat by at most three bytes from an 8 KiB cut.
const MAX_PREVIEW_PAGES: usize = MAX_REPORT_BYTES.div_ceil(PREVIEW_PAGE_BYTES - 3);

#[derive(Default)]
pub(crate) struct PreviewPages {
    ends: Vec<usize>,
    index: usize,
}
impl PreviewPages {
    pub(crate) fn reset(&mut self, text: &str) -> bool {
        self.index = 0;
        self.ends.clear();
        self.ends.push(0);
        if text.len() > MAX_REPORT_BYTES {
            self.ends.push(0);
            return false;
        }
        let mut start = 0;
        while start < text.len() {
            let mut end = (start + PREVIEW_PAGE_BYTES).min(text.len());
            while !text.is_char_boundary(end) {
                end -= 1;
            }
            self.ends.push(end);
            start = end;
        }
        if self.ends.len() == 1 {
            self.ends.push(0);
        }
        debug_assert!(self.count() <= MAX_PREVIEW_PAGES);
        true
    }
    pub(crate) fn index(&self) -> usize {
        self.index
    }
    pub(crate) fn count(&self) -> usize {
        self.ends.len().saturating_sub(1).max(1)
    }
    pub(crate) fn select(&mut self, index: usize) {
        self.index = index.min(self.count() - 1);
    }
    pub(crate) fn current<'a>(&self, text: &'a str) -> &'a str {
        let start = self.ends.get(self.index).copied().unwrap_or(0);
        let end = self.ends.get(self.index + 1).copied().unwrap_or(start);
        text.get(start..end).unwrap_or("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn utf8_pages_reconstruct_every_byte_without_splitting_characters() {
        for text in [
            String::new(),
            "a".repeat(8192),
            "a".repeat(8193),
            format!("{}한😀{}", "a".repeat(8191), "字".repeat(12000)),
            "😀".repeat(MAX_REPORT_BYTES / 4),
        ] {
            let mut pages = PreviewPages::default();
            assert!(pages.reset(&text));
            assert!(pages.count() <= MAX_PREVIEW_PAGES);
            let mut reconstructed = String::new();
            for index in 0..pages.count() {
                pages.select(index);
                let visible = pages.current(&text);
                assert!(visible.len() <= PREVIEW_PAGE_BYTES);
                reconstructed.push_str(visible);
            }
            assert_eq!(reconstructed, text);
            pages.select(usize::MAX);
            assert_eq!(pages.index, pages.count() - 1);
            assert!(!pages.reset(&"x".repeat(MAX_REPORT_BYTES + 1)));
            assert_eq!(pages.index, 0);
            assert_eq!(pages.current(&text), "");
        }
    }
}
