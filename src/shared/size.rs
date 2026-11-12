use std::fmt;

/// Disk size, displayed with decimal units like Finder and most file managers.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Bytes(pub u64);

impl std::ops::Add for Bytes {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0.saturating_add(other.0))
    }
}

impl std::iter::Sum for Bytes {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::default(), |total, size| total + size)
    }
}

impl fmt::Display for Bytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];

        if self.0 < 1000 {
            return write!(f, "{} B", self.0);
        }

        let mut value = self.0 as f64;
        let mut unit = 0;
        while value >= 1000.0 && unit < UNITS.len() - 1 {
            value /= 1000.0;
            unit += 1;
        }

        let rounded = (value * 10.0).round() / 10.0;
        if rounded.fract() == 0.0 {
            write!(f, "{rounded:.0} {}", UNITS[unit])
        } else {
            write!(f, "{rounded:.1} {}", UNITS[unit])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Bytes;

    #[test]
    fn formats_small_sizes_in_bytes() {
        assert_eq!(Bytes(0).to_string(), "0 B");
        assert_eq!(Bytes(999).to_string(), "999 B");
    }

    #[test]
    fn formats_with_decimal_units() {
        assert_eq!(Bytes(1_000).to_string(), "1 KB");
        assert_eq!(Bytes(1_500).to_string(), "1.5 KB");
        assert_eq!(Bytes(128_000_000).to_string(), "128 MB");
        assert_eq!(Bytes(6_420_000_000).to_string(), "6.4 GB");
    }

    #[test]
    fn caps_at_terabytes() {
        assert_eq!(Bytes(2_000_000_000_000_000).to_string(), "2000 TB");
    }

    #[test]
    fn sums_without_overflow() {
        let total: Bytes = [Bytes(u64::MAX), Bytes(1)].into_iter().sum();
        assert_eq!(total, Bytes(u64::MAX));
    }
}
