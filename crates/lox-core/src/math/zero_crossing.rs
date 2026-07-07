// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Zero-crossing classification for sampled scalar signals.

/// Direction of a zero-crossing of a scalar function between two samples.
///
/// Classification uses a half-open convention: a sample of `0.0` belongs to the
/// non-negative ("active") side, so a crossing is a transition between a
/// negative sample and a non-negative one. This matches the `value >= 0`
/// convention used by interval/event detection. `NaN` samples do not define a
/// direction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ZeroCrossing {
    /// The signal crosses from negative to non-negative.
    Up,
    /// The signal crosses from non-negative to negative.
    Down,
}

impl ZeroCrossing {
    /// Classifies the crossing direction between two consecutive samples `s0`
    /// and `s1`, returning `None` when there is no crossing or either sample is
    /// `NaN`.
    ///
    /// A value of `0.0` counts as the non-negative side, so brackets are
    /// half-open.
    pub fn new(s0: f64, s1: f64) -> Option<ZeroCrossing> {
        if s0.is_nan() || s1.is_nan() {
            return None;
        }
        match (s0 < 0.0, s1 < 0.0) {
            (true, false) => Some(ZeroCrossing::Up),
            (false, true) => Some(ZeroCrossing::Down),
            _ => None,
        }
    }
}

impl core::fmt::Display for ZeroCrossing {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ZeroCrossing::Up => write!(f, "up"),
            ZeroCrossing::Down => write!(f, "down"),
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use super::*;

    #[test]
    fn test_zero_crossing() {
        // Negative -> positive is Up; positive -> negative is Down.
        assert_eq!(ZeroCrossing::new(-1.0, 1.0), Some(ZeroCrossing::Up));
        assert_eq!(ZeroCrossing::new(1.0, -1.0), Some(ZeroCrossing::Down));
        // Same side -> no crossing.
        assert_eq!(ZeroCrossing::new(-1.0, -2.0), None);
        assert_eq!(ZeroCrossing::new(1.0, 2.0), None);
        // Half-open: zero counts as the non-negative side.
        assert_eq!(ZeroCrossing::new(-1.0, 0.0), Some(ZeroCrossing::Up));
        assert_eq!(ZeroCrossing::new(0.0, -1.0), Some(ZeroCrossing::Down));
        assert_eq!(ZeroCrossing::new(0.0, 1.0), None);
        assert_eq!(ZeroCrossing::new(1.0, 0.0), None);
        // The sign of zero does not affect classification.
        assert_eq!(ZeroCrossing::new(-1.0, -0.0), Some(ZeroCrossing::Up));
        assert_eq!(ZeroCrossing::new(-0.0, -1.0), Some(ZeroCrossing::Down));
        // NaN never defines a direction.
        assert_eq!(ZeroCrossing::new(f64::NAN, 1.0), None);
        assert_eq!(ZeroCrossing::new(-1.0, f64::NAN), None);
    }

    #[test]
    fn test_zero_crossing_display() {
        assert_eq!(ZeroCrossing::Up.to_string(), "up");
        assert_eq!(ZeroCrossing::Down.to_string(), "down");
    }
}
