//! # GTFS Currency Amounts
//!
//! The spec's "Currency amount" field type: a decimal value whose
//! number of decimal places is specified by ISO 4217 for the
//! accompanying currency code. The spec mandates that financial
//! calculations be processed as decimal rather than floating point,
//! so this crate stores amounts exactly as an integer mantissa and a
//! decimal scale instead of `f64`.

use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};

use crate::error::GtfsError;

/// An exact decimal currency amount (spec type "Currency amount").
///
/// The value is `mantissa / 10^scale`: `CurrencyAmount::new(5700, 2)`
/// is `57.00`. Comparisons are numeric, so `1.5 == 1.50`; `Display`
/// preserves the original scale. Negative amounts are allowed (the
/// spec uses them e.g. for transfer discounts).
///
/// # Examples
///
/// ```
/// fn main() -> Result<(), gtfs_rs::GtfsError> {
///     use gtfs_rs::CurrencyAmount;
///
///     let price = CurrencyAmount::parse("57.00")?;
///     assert_eq!(price, CurrencyAmount::new(5700, 2)?);
///     assert_eq!(price.to_string(), "57.00");
///     assert!(price < CurrencyAmount::parse("57.10")?);
///     assert_eq!(CurrencyAmount::parse("1.5")?, CurrencyAmount::parse("1.50")?);
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone, Copy)]
pub struct CurrencyAmount {
    /// Integer mantissa; the amount is `mantissa / 10^scale`
    mantissa: i64,
    /// Number of decimal places, at most [`CurrencyAmount::MAX_SCALE`]
    scale: u8,
}

impl CurrencyAmount {
    /// Maximum supported number of decimal places.
    pub const MAX_SCALE: u8 = 18;

    /// Creates an amount from an integer mantissa and a decimal
    /// scale: `new(5700, 2)` is `57.00`.
    ///
    /// # Arguments
    ///
    /// * `mantissa` - Integer mantissa, may be negative
    /// * `scale` - Number of decimal places
    ///
    /// # Errors
    ///
    /// Returns [`GtfsError::InvalidCurrencyAmount`] if `scale`
    /// exceeds [`CurrencyAmount::MAX_SCALE`].
    ///
    /// # Examples
    ///
    /// ```
    /// fn main() -> Result<(), gtfs_rs::GtfsError> {
    ///     use gtfs_rs::CurrencyAmount;
    ///
    ///     let fare = CurrencyAmount::new(250, 2)?;
    ///     assert_eq!(fare.to_string(), "2.50");
    ///     let discount = CurrencyAmount::new(-75, 2)?;
    ///     assert_eq!(discount.to_string(), "-0.75");
    ///     Ok(())
    /// }
    /// ```
    pub fn new(mantissa: i64, scale: u8) -> Result<Self, GtfsError> {
        if scale > Self::MAX_SCALE {
            return Err(GtfsError::InvalidCurrencyAmount {
                value: format!("{}e-{}", mantissa, scale),
            });
        }
        Ok(CurrencyAmount { mantissa, scale })
    }

    /// Parses a decimal string such as `"57"`, `"57.00"` or
    /// `"-2.50"`.
    ///
    /// # Arguments
    ///
    /// * `value` - Decimal string, optionally with a leading `-`
    ///
    /// # Errors
    ///
    /// Returns [`GtfsError::InvalidCurrencyAmount`] if the string is
    /// not a decimal number, has more than
    /// [`CurrencyAmount::MAX_SCALE`] decimal places, or does not fit
    /// the internal 64-bit mantissa.
    ///
    /// # Examples
    ///
    /// ```
    /// fn main() -> Result<(), gtfs_rs::GtfsError> {
    ///     use gtfs_rs::CurrencyAmount;
    ///
    ///     assert_eq!(CurrencyAmount::parse("2.50")?.mantissa(), 250);
    ///     assert_eq!(CurrencyAmount::parse("2.50")?.scale(), 2);
    ///     assert_eq!(CurrencyAmount::parse("-0.75")?.to_string(), "-0.75");
    ///     assert!(CurrencyAmount::parse("1,50").is_err());
    ///     Ok(())
    /// }
    /// ```
    pub fn parse(value: &str) -> Result<Self, GtfsError> {
        let invalid = || GtfsError::InvalidCurrencyAmount {
            value: value.to_string(),
        };
        let v = value.trim();
        let (negative, digits) = match v.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, v),
        };
        let (int_part, frac_part) = match digits.split_once('.') {
            Some((int_part, frac_part)) => (int_part, frac_part),
            None => (digits, ""),
        };
        let all_digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
        if !all_digits(int_part) || (digits.contains('.') && !all_digits(frac_part)) {
            return Err(invalid());
        }
        if frac_part.len() > usize::from(Self::MAX_SCALE) {
            return Err(invalid());
        }
        let mantissa: i64 = format!("{int_part}{frac_part}")
            .parse()
            .map_err(|_| invalid())?;
        Ok(CurrencyAmount {
            mantissa: if negative { -mantissa } else { mantissa },
            scale: frac_part.len() as u8,
        })
    }

    /// Returns the integer mantissa; the amount is
    /// `mantissa / 10^scale`.
    pub fn mantissa(&self) -> i64 {
        self.mantissa
    }

    /// Returns the number of decimal places.
    pub fn scale(&self) -> u8 {
        self.scale
    }

    /// Returns the amount as an `f64` approximation. Convenient for
    /// modeling; do not use for financial calculations.
    ///
    /// # Examples
    ///
    /// ```
    /// fn main() -> Result<(), gtfs_rs::GtfsError> {
    ///     use gtfs_rs::CurrencyAmount;
    ///
    ///     assert_eq!(CurrencyAmount::parse("2.50")?.to_f64(), 2.5);
    ///     Ok(())
    /// }
    /// ```
    pub fn to_f64(&self) -> f64 {
        self.mantissa as f64 / 10f64.powi(i32::from(self.scale))
    }

    /// Returns the mantissa and scale with trailing zeros removed,
    /// so numerically equal amounts normalize identically.
    fn normalized(&self) -> (i64, u8) {
        let mut mantissa = self.mantissa;
        let mut scale = self.scale;
        if mantissa == 0 {
            return (0, 0);
        }
        while scale > 0 && mantissa % 10 == 0 {
            mantissa /= 10;
            scale -= 1;
        }
        (mantissa, scale)
    }
}

impl PartialEq for CurrencyAmount {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for CurrencyAmount {}

impl PartialOrd for CurrencyAmount {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CurrencyAmount {
    fn cmp(&self, other: &Self) -> Ordering {
        // Cross-multiplied comparison in i128: |mantissa| <= ~9.2e18
        // and scale <= 18, so the products stay within i128.
        let lhs = i128::from(self.mantissa) * 10i128.pow(u32::from(other.scale));
        let rhs = i128::from(other.mantissa) * 10i128.pow(u32::from(self.scale));
        lhs.cmp(&rhs)
    }
}

impl Hash for CurrencyAmount {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let (mantissa, scale) = self.normalized();
        mantissa.hash(state);
        scale.hash(state);
    }
}

impl fmt::Display for CurrencyAmount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.scale == 0 {
            return write!(f, "{}", self.mantissa);
        }
        let sign = if self.mantissa < 0 { "-" } else { "" };
        let abs = i128::from(self.mantissa).unsigned_abs();
        let divisor = 10u128.pow(u32::from(self.scale));
        write!(
            f,
            "{}{}.{:0width$}",
            sign,
            abs / divisor,
            abs % divisor,
            width = usize::from(self.scale)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_and_display() -> Result<(), GtfsError> {
        assert_eq!(CurrencyAmount::parse("57")?.to_string(), "57");
        assert_eq!(CurrencyAmount::parse("57.00")?.to_string(), "57.00");
        assert_eq!(CurrencyAmount::parse("0.05")?.to_string(), "0.05");
        assert_eq!(CurrencyAmount::parse("-2.50")?.to_string(), "-2.50");
        assert_eq!(CurrencyAmount::parse(" 1.5 ")?.to_string(), "1.5");
        Ok(())
    }

    #[test]
    fn test_parse_rejects_garbage() {
        for bad in ["", ".", "1.", ".5", "1.2.3", "1,50", "abc", "--1", "1e3"] {
            assert!(CurrencyAmount::parse(bad).is_err(), "accepted {:?}", bad);
        }
        // more decimal places than MAX_SCALE
        assert!(CurrencyAmount::parse("0.0000000000000000001").is_err());
        // mantissa overflow
        assert!(CurrencyAmount::parse("92233720368547758080").is_err());
    }

    #[test]
    fn test_equality_across_scales() -> Result<(), GtfsError> {
        assert_eq!(
            CurrencyAmount::parse("1.5")?,
            CurrencyAmount::parse("1.50")?
        );
        assert_eq!(CurrencyAmount::parse("0")?, CurrencyAmount::parse("0.00")?);
        assert_ne!(
            CurrencyAmount::parse("1.5")?,
            CurrencyAmount::parse("1.05")?
        );
        Ok(())
    }

    #[test]
    fn test_ordering() -> Result<(), GtfsError> {
        assert!(CurrencyAmount::parse("2.50")? < CurrencyAmount::parse("10")?);
        assert!(CurrencyAmount::parse("-0.01")? < CurrencyAmount::parse("0")?);
        // exactness: 0.1 + 0.2 style values compare exactly
        assert!(CurrencyAmount::parse("0.30")? > CurrencyAmount::parse("0.29999")?);
        Ok(())
    }

    #[test]
    fn test_new_scale_cap() {
        assert!(CurrencyAmount::new(1, 19).is_err());
        assert!(CurrencyAmount::new(1, 18).is_ok());
    }
}
