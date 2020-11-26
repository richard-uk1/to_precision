//! A function that works like javascript's `toPreceision`.
//!
//! Internally it rounds and then uses the build-in algorithm, so it will give different results to
//! `toPrecision`. They may converge over time.
use std::fmt;

pub trait FloatExt {
    type Display: fmt::Display;
    fn to_precision(self, p: u8) -> Self::Display;
}

const MAX_FRACTION_DIGITS: u8 = 21;

impl FloatExt for f64 {
    type Display = F64Display;
    fn to_precision(self, p: u8) -> Self::Display {
        assert!(
            1 <= p && p <= MAX_FRACTION_DIGITS,
            "precision must satisfy 1 <= p ({}) <= {}",
            p,
            MAX_FRACTION_DIGITS
        );
        F64Display(self, p.into())
    }
}

// u16 should be big enough for the exponent/precision
#[derive(Debug)]
pub struct F64Display(f64, i32);

impl fmt::Display for F64Display {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut x = self.0;

        if x.is_nan() {
            return write!(f, "NaN");
        }
        if x == 0. {
            return write!(f, "0");
        }
        if x < 0. {
            x = -x;
            write!(f, "-")?;
        }
        if !x.is_finite() {
            return write!(f, "∞");
        }
        // round and defer to std impl
        write!(f, "{}", to_sig_figs(self.0, self.1))
    }
}

/// Round the number to the given significant figures.
fn to_sig_figs(x: f64, sf: i32) -> f64 {
    println!("to_sig_figs({}, {})", x, sf);
    let e = ten_power_leq(x);
    println!("e = {}", e);
    // two branches depending on the sign of e - sf + 1
    // We need this to combat fp error: although e.g. 0.1 is representable in fp, we won't get that
    // answer when doing 10000 * 0.000001.
    let p = e - sf + 1;
    if p < 0 {
        let tens = (10.0f64).powi(-p);
        (x * tens).round() / tens
    } else {
        let tens = (10.0f64).powi(p);
        (x / tens).round() * tens
    }
}

/// Return integer e such that `10^e <= x < 10^(e+1)`
fn ten_power_leq(x: f64) -> i32 {
    assert!(x != 0., "power of 10 only makes sense on nonzero numbers");
    let x = x.abs();
    let log10 = x.log10().floor();
    debug_assert!(i32::MIN as f64 <= log10 && log10 <= i32::MAX as f64);
    debug_assert_eq!(log10 as i32 as f64, log10);
    let log10 = log10 as i32;
    // we might be off by 1 because of fp precicision errors.
    if 10.0f64.powi(log10 + 1) < x {
        log10 + 1
    } else {
        log10
    }
}

#[cfg(test)]
mod tests {
    use super::FloatExt as _;
    use std::f64;

    #[test]
    fn ten_power_leq() {
        for (input, output) in vec![(1., 0), (-1., 0), (0.1, -1), (0.99999999999, -1)] {
            assert_eq!(super::ten_power_leq(input,), output);
        }
    }

    #[test]
    fn to_sig_figs() {
        for (x, sf, expected) in vec![
            (1., 3, 1.),
            (100., 3, 100.),
            (1234., 3, 1230.),
            (9999., 4, 9999.),
            (9999., 3, 10_000.),
            (9999., 1, 10_000.),
            (0.1, 3, 0.1),
            (0.1234, 3, 0.123),
        ] {
            assert_eq!(
                super::to_sig_figs(x, sf),
                expected,
                "to_sig_figs({}, {}) = {}, {}",
                x,
                sf,
                super::to_sig_figs(x, sf),
                expected
            );
        }
    }

    #[test]
    #[should_panic]
    fn bad_precision() {
        1.0f64.to_precision(0);
    }

    #[test]
    fn it_works() {
        for (input, sf, expected) in vec![
            (f64::NAN, 3, "NaN"),
            (f64::INFINITY, 3, "∞"),
            (f64::NEG_INFINITY, 3, "-∞"),
            (0., 3, "0"),
            (-0., 3, "0"),
            (0.999, 3, "0.999"),
            (0.9999, 3, "1"),
            (0.7000000000000002, 5, "0.7"),
            (f64::from_bits(4603579539098121012), 4, "0.6"),
        ] {
            assert_eq!(input.to_precision(sf).to_string(), expected);
        }
    }
}
