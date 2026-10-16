//! Bounded decimal transfer-rate labels shared by download and network UI.
use crate::catalog as c;

pub(crate) fn format_bytes_per_second(rate: Option<f64>) -> String {
    let Some(rate) = rate.filter(|rate| rate.is_finite() && *rate >= 0.0) else {
        return c::UNKNOWN.to_owned();
    };
    if rate == 0.0 {
        return format!("0 {}", c::RATE_BPS);
    }
    // Bound both formatting work and label width, without multiplying huge f64 values.
    if rate >= 1_000_000_000_000.0 {
        return format!("≥ 1000 {}", c::RATE_GBPS);
    }
    let units = [c::RATE_BPS, c::RATE_KBPS, c::RATE_MBPS, c::RATE_GBPS];
    let mut scaled = rate;
    let mut unit = 0;
    while scaled >= 1000.0 && unit < units.len() - 1 {
        scaled /= 1000.0;
        unit += 1;
    }
    let mut rounded = (scaled * 10.0).round() / 10.0;
    if rounded >= 1000.0 && unit < units.len() - 1 {
        unit += 1;
        rounded = (scaled / 1000.0 * 10.0).round() / 10.0;
    }
    if rounded == 0.0 {
        return format!("< 0.1 {}", c::RATE_BPS);
    }
    format!("{rounded:.1} {}", units[unit])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decimal_units_round_and_carry_without_losing_unknown_or_zero() {
        for (rate, expected) in [
            (0.0, "0 B/s"),
            (-0.0, "0 B/s"),
            (0.001, "< 0.1 B/s"),
            (0.05, "0.1 B/s"),
            (999.94, "999.9 B/s"),
            (999.95, "1.0 KB/s"),
            (1000.0, "1.0 KB/s"),
            (999_940.0, "999.9 KB/s"),
            (999_950.0, "1.0 MB/s"),
            (1_000_000.0, "1.0 MB/s"),
            (999_950_000.0, "1.0 GB/s"),
            (1_000_000_000.0, "1.0 GB/s"),
            (1_000_000_000_000.0, "≥ 1000 GB/s"),
            (f64::MAX, "≥ 1000 GB/s"),
        ] {
            assert_eq!(format_bytes_per_second(Some(rate)), expected, "{rate}");
        }
        for rate in [
            None,
            Some(-1.0),
            Some(f64::NAN),
            Some(f64::INFINITY),
            Some(f64::NEG_INFINITY),
        ] {
            assert_eq!(format_bytes_per_second(rate), c::UNKNOWN);
        }
    }
}
