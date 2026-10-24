use std::time::Duration;

// Zero, 1000 upper-bound buckets of 100 us, and an overflow bucket.
// The overflow bucket reports the observed maximum, never an invented cap.
const BUCKETS: usize = 1002;
const WIDTH_NS: u128 = 100_000;
pub(crate) struct LatenessHistogram {
    counts: [u64; BUCKETS],
    samples: u64,
    maximum: Duration,
}
impl LatenessHistogram {
    pub(crate) fn new() -> Self {
        Self {
            counts: [0; BUCKETS],
            samples: 0,
            maximum: Duration::ZERO,
        }
    }
    pub(crate) fn record(&mut self, value: Duration) {
        // The engine checks its attempt counter before recording each sample.
        // Keep this helper bounded even if that caller contract changes.
        if self.samples == u64::MAX {
            return;
        }
        let bucket = value
            .as_nanos()
            .div_ceil(WIDTH_NS)
            .min((BUCKETS - 1) as u128) as usize;
        self.counts[bucket] += 1;
        self.samples += 1;
        self.maximum = self.maximum.max(value);
    }
    pub(crate) fn percentile(&self, percentile: u32) -> Duration {
        if self.samples == 0 {
            return Duration::ZERO;
        }
        let rank = (u128::from(self.samples) * u128::from(percentile.clamp(1, 100))).div_ceil(100);
        let mut cumulative = 0u128;
        for (bucket, count) in self.counts.iter().enumerate() {
            cumulative += u128::from(*count);
            if cumulative >= rank {
                return if bucket == BUCKETS - 1 {
                    self.maximum
                } else {
                    Duration::from_nanos((bucket as u128 * WIDTH_NS) as u64).min(self.maximum)
                };
            }
        }
        self.maximum
    }
    pub(crate) fn publish(&self, metrics: &mut crate::Metrics) {
        metrics.lateness_p50 = self.percentile(50);
        metrics.lateness_p95 = self.percentile(95);
        metrics.lateness_p99 = self.percentile(99);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_rank_keeps_rare_outlier_out_of_p99() {
        let mut histogram = LatenessHistogram::new();
        for _ in 0..50 {
            histogram.record(Duration::ZERO);
        }
        for _ in 0..49 {
            histogram.record(Duration::from_micros(123));
        }
        histogram.record(Duration::from_secs(2));
        assert_eq!(histogram.percentile(50), Duration::ZERO);
        assert_eq!(histogram.percentile(95), Duration::from_micros(200));
        assert_eq!(histogram.percentile(99), Duration::from_micros(200));
        assert_eq!(histogram.percentile(100), Duration::from_secs(2));
    }

    #[test]
    fn bucket_edges_are_conservative_and_never_exceed_observed_maximum() {
        for micros in [1, 99, 100, 101, 99_999, 100_000, 100_001] {
            let mut histogram = LatenessHistogram::new();
            histogram.record(Duration::from_micros(micros));
            assert_eq!(histogram.percentile(99), Duration::from_micros(micros));
        }
        let mut histogram = LatenessHistogram::new();
        histogram.record(Duration::from_nanos(100_001));
        histogram.record(Duration::from_micros(500));
        assert_eq!(histogram.percentile(50), Duration::from_micros(200));
    }

    #[test]
    fn empty_and_extreme_durations_are_defined() {
        let mut histogram = LatenessHistogram::new();
        assert_eq!(histogram.percentile(99), Duration::ZERO);
        histogram.record(Duration::MAX);
        assert_eq!(histogram.percentile(99), Duration::MAX);
    }
}
