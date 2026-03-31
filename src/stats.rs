use serde::Serialize;

/// Raw timing samples collected from repeated runs (all in seconds).
pub struct Samples {
    // kept in insertion order; sorted lazily when needed
    data: Vec<f64>,
}

impl Samples {
    pub fn new(data: Vec<f64>) -> Self {
        Self { data }
    }

    fn sorted(&self) -> Vec<f64> {
        let mut v = self.data.clone();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        v
    }

    pub fn count(&self) -> usize {
        self.data.len()
    }

    pub fn mean(&self) -> f64 {
        if self.data.is_empty() {
            return 0.0;
        }
        self.data.iter().sum::<f64>() / self.data.len() as f64
    }

    /// Sample standard deviation (Bessel-corrected, N-1 denominator).
    pub fn stddev(&self) -> f64 {
        let n = self.data.len();
        if n < 2 {
            return 0.0;
        }
        let m = self.mean();
        let variance = self.data.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (n - 1) as f64;
        variance.sqrt()
    }

    pub fn min(&self) -> f64 {
        self.data.iter().cloned().fold(f64::INFINITY, f64::min)
    }

    pub fn max(&self) -> f64 {
        self.data.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
    }

    /// Linear interpolation percentile (same method as numpy's default).
    pub fn percentile(&self, p: f64) -> f64 {
        assert!((0.0..=100.0).contains(&p));
        let sorted = self.sorted();
        let n = sorted.len();
        if n == 1 {
            return sorted[0];
        }

        let idx = (p / 100.0) * (n - 1) as f64;
        let lo = idx.floor() as usize;
        let hi = idx.ceil() as usize;
        let frac = idx - lo as f64;

        sorted[lo] + frac * (sorted[hi] - sorted[lo])
    }

    pub fn summarise(&self) -> Summary {
        Summary {
            n: self.count(),
            mean: self.mean(),
            stddev: self.stddev(),
            min: self.min(),
            max: self.max(),
            p50: self.percentile(50.0),
            p95: self.percentile(95.0),
            p99: self.percentile(99.0),
        }
    }
}

/// All the stats we care about for one benchmark run.
#[derive(Debug, Serialize, Clone)]
pub struct Summary {
    pub n: usize,
    pub mean: f64,
    pub stddev: f64,
    pub min: f64,
    pub max: f64,
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
}

impl Summary {
    /// Relative standard deviation as a percentage — useful sanity check.
    pub fn rsd_pct(&self) -> f64 {
        if self.mean == 0.0 {
            return 0.0;
        }
        (self.stddev / self.mean) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mean_of_known_sequence() {
        let s = Samples::new(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        assert!((s.mean() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn stddev_population_one() {
        let s = Samples::new(vec![42.0]);
        assert_eq!(s.stddev(), 0.0);
    }

    #[test]
    fn percentile_median() {
        let s = Samples::new(vec![10.0, 20.0, 30.0, 40.0, 50.0]);
        assert!((s.percentile(50.0) - 30.0).abs() < 1e-10);
    }

    #[test]
    fn percentile_p99_small_set() {
        let s = Samples::new(vec![1.0, 2.0, 3.0, 4.0, 100.0]);
        // p99 should be very close to the outlier
        assert!(s.percentile(99.0) > 90.0);
    }

    #[test]
    fn min_max() {
        let s = Samples::new(vec![5.0, 3.0, 9.0, 1.0, 7.0]);
        assert_eq!(s.min(), 1.0);
        assert_eq!(s.max(), 9.0);
    }
}