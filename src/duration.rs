use std::str::FromStr;
use std::time::Duration;

use failure::Error;
use nom::digit;
use nom::digit1;
use nom::types::CompleteStr;

fn to_nanos(s: CompleteStr) -> Result<u32, ()> {
    let s = s.0;

    const NANO_DIGITS: usize = 9;

    if s.len() > NANO_DIGITS {
        return Err(());
    }

    let extra_zeros = (NANO_DIGITS - s.len()) as u32;
    let mul = 10u32.pow(extra_zeros);
    let num = u32::from_str(s).map_err(|_| ())?;
    Ok(num * mul)
}

named!(period_num<CompleteStr, (u64, Option<CompleteStr>)>,
    do_parse!(
        whole: num >>
        rest: opt!(preceded!(tag!("."), digit1)) >>
        (( whole, rest ))
    )
);

fn to_u64(s: CompleteStr) -> Result<u64, ()> {
    s.0.parse().map_err(|_| ())
}

named!(num<CompleteStr, u64>,
    map_res!(digit1, to_u64));

named!(time<CompleteStr, Duration>,
    do_parse!(
        tag!("T") >>
        h: opt!(terminated!(num, tag!("H"))) >>
        m: opt!(terminated!(num, tag!("M"))) >>
        s: opt!(terminated!(period_num, tag!("S"))) >>
        ( unimplemented!() )
    ));

named!(period<CompleteStr, Duration>,
    do_parse!(
        sign: opt!(tag!("-")) >>
        tag!("P") >>
        w: opt!(terminated!(num, tag!("W"))) >>
        d: opt!(terminated!(num, tag!("D"))) >>
        ( unimplemented!() )
    ));

fn parse(input: &str) -> Result<Duration, Error> {
    let input = CompleteStr(input);
    unimplemented!()
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
