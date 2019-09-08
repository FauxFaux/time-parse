mod duration_hand;
mod duration_nom;

use std::str::FromStr;

use failure::Error;

const SECS_PER_MINUTE: u64 = 60;
const SECS_PER_HOUR: u64 = 60 * SECS_PER_MINUTE;
const SECS_PER_DAY: u64 = 24 * SECS_PER_HOUR;
const SECS_PER_WEEK: u64 = 7 * SECS_PER_DAY;

pub use self::duration_hand::parse;
pub use self::duration_nom::parse as parse_nom;

/// AsRef<str> but implementable on nom types
/// Workaround for https://github.com/Geal/nom/pull/753
trait Strable {
    fn as_str(&self) -> &str;
}

impl<'s> Strable for &'s str {
    fn as_str(&self) -> &str {
        self
    }
}

impl<'s> Strable for dyn AsRef<str> {
    fn as_str(&self) -> &str {
        self.as_ref()
    }
}

impl<'s> Strable for ::nom::types::CompleteStr<'s> {
    fn as_str(&self) -> &str {
        self.0
    }
}

fn to_nanos<S: Strable>(s: S) -> Result<u32, Error> {
    let s = s.as_str();

    const NANO_DIGITS: usize = 9;
    ensure!(
        s.len() <= NANO_DIGITS,
        "too many nanoseconds digits: {:?}",
        s
    );

    let extra_zeros = (NANO_DIGITS - s.len()) as u32;
    let mul = 10u32.pow(extra_zeros);
    let num = u32::from_str(s)?;
    Ok(num * mul)
}

#[cfg(test)]
mod tests {
    use nom::types::CompleteStr;

    #[test]
    fn test_nanos() {
        use super::to_nanos;
        assert_eq!(0, to_nanos(CompleteStr("0")).unwrap());
        assert_eq!(0, to_nanos(CompleteStr("000")).unwrap());

        assert_eq!(1, to_nanos(CompleteStr("000000001")).unwrap());
        assert_eq!(10, to_nanos(CompleteStr("00000001")).unwrap());
        assert_eq!(100, to_nanos(CompleteStr("0000001")).unwrap());
        assert_eq!(1000, to_nanos(CompleteStr("000001")).unwrap());
        assert_eq!(10000, to_nanos(CompleteStr("00001")).unwrap());
        assert_eq!(100000, to_nanos(CompleteStr("0001")).unwrap());
        assert_eq!(1000000, to_nanos(CompleteStr("001")).unwrap());
        assert_eq!(10000000, to_nanos(CompleteStr("01")).unwrap());
        assert_eq!(100000000, to_nanos(CompleteStr("1")).unwrap());

        assert_eq!(7_010, to_nanos(CompleteStr("00000701")).unwrap());
    }
}
