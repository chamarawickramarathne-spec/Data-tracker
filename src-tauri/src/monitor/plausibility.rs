const DEFAULT_LINK_BPS: u64 = 1_000_000_000;
const TOLERANCE: f64 = 1.10;
const FLOOR_BYTES: f64 = 1_000_000.0;

pub fn is_plausible_delta(delta: u64, link_bps: u64, elapsed_secs: f64) -> bool {
    if delta == 0 {
        return true;
    }
    if !elapsed_secs.is_finite() || elapsed_secs <= 0.0 {
        return false;
    }

    let link_bps = if link_bps == 0 {
        DEFAULT_LINK_BPS
    } else {
        link_bps
    };
    let max_bytes = (link_bps as f64 / 8.0) * elapsed_secs * TOLERANCE + FLOOR_BYTES;
    (delta as f64) <= max_bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_spike() {
        assert!(!is_plausible_delta(43_825_725_950_298, 1_000_000_000, 3.0));
    }

    #[test]
    fn accepts_traffic_within_link_capacity() {
        assert!(is_plausible_delta(8_000_000_000, 1_000_000_000, 60.0));
        assert!(is_plausible_delta(2_000_000, 0, 3.0));
    }

    #[test]
    fn rejects_impossible_traffic() {
        assert!(!is_plausible_delta(10_000_000_000, 1_000_000_000, 60.0));
    }

    #[test]
    fn handles_zero_and_invalid_elapsed() {
        assert!(is_plausible_delta(0, 1_000_000_000, 0.0));
        assert!(!is_plausible_delta(1, 1_000_000_000, 0.0));
        assert!(!is_plausible_delta(1, 1_000_000_000, f64::NAN));
    }
}
