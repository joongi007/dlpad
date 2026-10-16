//! UI-independent millisecond selection and playhead bounds.
use crate::options::ValidationError;
pub const MAX_DURATION_MS: u64 = 7 * 24 * 60 * 60 * 1000;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MediaSelection {
    duration_ms: u64,
    start_ms: u64,
    end_ms: u64,
    playhead_ms: u64,
}
impl MediaSelection {
    pub fn new(duration_ms: u64) -> Result<Self, ValidationError> {
        if duration_ms == 0 || duration_ms > MAX_DURATION_MS {
            return Err(ValidationError("영상 길이가 유효하지 않습니다"));
        }
        Ok(Self {
            duration_ms,
            start_ms: 0,
            end_ms: duration_ms,
            playhead_ms: 0,
        })
    }
    pub fn duration_ms(self) -> u64 {
        self.duration_ms
    }
    pub fn start_ms(self) -> u64 {
        self.start_ms
    }
    pub fn end_ms(self) -> u64 {
        self.end_ms
    }
    pub fn playhead_ms(self) -> u64 {
        self.playhead_ms
    }
    pub fn is_full(self) -> bool {
        self.start_ms == 0 && self.end_ms == self.duration_ms
    }
    pub fn set_range(&mut self, start_ms: u64, end_ms: u64) -> Result<(), ValidationError> {
        if start_ms >= end_ms || end_ms > self.duration_ms {
            return Err(ValidationError("선택 구간이 영상 길이를 벗어났습니다"));
        }
        self.start_ms = start_ms;
        self.end_ms = end_ms;
        Ok(())
    }
    pub fn move_start(&mut self, value: u64) {
        self.start_ms = value.min(self.end_ms - 1);
    }
    pub fn move_end(&mut self, value: u64) {
        self.end_ms = value.clamp(self.start_ms + 1, self.duration_ms);
    }
    pub fn seek(&mut self, value: u64) {
        self.playhead_ms = value.min(self.duration_ms.saturating_sub(1));
    }
    pub fn select_all(&mut self) {
        self.start_ms = 0;
        self.end_ms = self.duration_ms;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_manual_range_preserves_original_and_playhead_is_independent() {
        let mut s = MediaSelection::new(28_800_000).unwrap();
        s.set_range(1000, 9000).unwrap();
        s.seek(12000);
        let old = s;
        for (a, b) in [(0, 0), (10000, 9000), (0, 28_800_001), (u64::MAX, u64::MAX)] {
            assert!(s.set_range(a, b).is_err());
            assert_eq!(s, old);
        }
        s.move_start(u64::MAX);
        assert_eq!(s.start_ms(), 8999);
        s.move_end(0);
        assert_eq!(s.end_ms(), 9000);
        assert_eq!(s.playhead_ms(), 12000);
        s.select_all();
        assert!(s.is_full());
        assert_eq!(s.playhead_ms(), 12000);
    }
    #[test]
    fn unknown_excessive_duration_and_very_short_media() {
        assert!(MediaSelection::new(0).is_err());
        assert!(MediaSelection::new(MAX_DURATION_MS + 1).is_err());
        let mut s = MediaSelection::new(1).unwrap();
        s.move_start(20);
        s.move_end(0);
        s.seek(9);
        assert_eq!((s.start_ms(), s.end_ms(), s.playhead_ms()), (0, 1, 0));
    }
}
