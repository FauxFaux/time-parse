use std::iter::Peekable;
use std::str::FromStr;
use std::time::Duration;

use failure::Error;
use nom::digit1;
use nom::types::CompleteStr;

const SECS_PER_MINUTE: u64 = 60;
const SECS_PER_HOUR: u64 = 60 * SECS_PER_MINUTE;
const SECS_PER_DAY: u64 = 24 * SECS_PER_HOUR;
const SECS_PER_WEEK: u64 = 7 * SECS_PER_DAY;

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

named!(period_num<CompleteStr, (u64, u32)>,
    do_parse!(
        whole: num >>
        rest: opt!(preceded!(tag!("."), map_res!(digit1, to_nanos))) >>
        (( whole, rest.unwrap_or(0) ))
    )
);

fn to_u64(s: CompleteStr) -> Result<u64, ()> {
    s.0.parse().map_err(|_| ())
}

named!(num<CompleteStr, u64>,
    map_res!(digit1, to_u64));

named!(time<CompleteStr, (u64, u32)>,
    do_parse!(
        tag!("T") >>
        h: opt!(terminated!(num, tag!("H"))) >>
        m: opt!(terminated!(num, tag!("M"))) >>
        s: opt!(terminated!(period_num, tag!("S"))) >>
        (
          (  h.unwrap_or(0) * SECS_PER_HOUR
           + m.unwrap_or(0) * SECS_PER_MINUTE
           + s.map(|(s, _ns)|  s).unwrap_or(0),
             s.map(|(_s, ns)| ns).unwrap_or(0)
          )
        )
    ));

named!(period<CompleteStr, (u64, u32)>,
    do_parse!(
        tag!("P") >>
        w: opt!(terminated!(num, tag!("W"))) >>
        d: opt!(terminated!(num, tag!("D"))) >>
        t: opt!(time) >>
        (
          (  w.unwrap_or(0) * SECS_PER_WEEK
           + d.unwrap_or(0) * SECS_PER_DAY
           + t.map(|(s, _ns)|  s).unwrap_or(0),
             t.map(|(_s, ns)| ns).unwrap_or(0)
          )
        )
    ));

pub fn parse_nom(input: &str) -> Result<Duration, Error> {
    match period(CompleteStr(input)) {
        Ok((CompleteStr(""), (s, ns))) => Ok(Duration::new(s, ns)),
        other => bail!("invalid: {:?}", other),
    }
}

struct Parts<'s> {
    inner: &'s str,
}

impl<'s> Parts<'s> {
    fn new(inner: &str) -> Parts {
        Parts { inner }
    }
}

impl<'s> Iterator for Parts<'s> {
    type Item = (&'s str, char);

    fn next(&mut self) -> Option<(&'s str, char)> {
        self.inner
            .find(|c: char| c.is_ascii_alphabetic())
            .map(|next| {
                let (init, point) = self.inner.split_at(next);
                self.inner = &point[1..];
                (init, point.chars().next().unwrap())
            })
    }
}

pub fn parse(input: &str) -> Result<Duration, Error> {
    let mut parts = input.match_indices(|c: char| c.is_alphabetic()).peekable();

    let (sign, mut last) = match parts.next() {
        Some((0, "p")) | Some((0, "P")) => (1, 1),
        Some((1, "p")) | Some((1, "P")) if input.starts_with('-') => (-1, 2),
        _ => bail!("invalid prefix, expecting 'P' or '-P'"),
    };

    let mut seconds = 0u64;
    let mut nanos = 0u32;

    for (token, parse_nanos, mul) in &[
        ("W", false, Some(SECS_PER_WEEK)),
        ("D", false, Some(SECS_PER_DAY)),
        ("T", false, None),
        ("H", false, Some(SECS_PER_HOUR)),
        ("M", false, Some(SECS_PER_MINUTE)),
        ("S", true, Some(1)),
    ] {
        if let Some((idx, available_token)) = parts.peek().cloned() {
            if !token.eq_ignore_ascii_case(available_token) {
                continue;
            }

            parts.next().unwrap();

            let mut body = &input[last..idx];

            if let Some(mul) = mul {
                if *parse_nanos {
                    if let Some(first_point) = body.find('.') {
                        let (main, after) = body.split_at(first_point);
                        body = main;
                        nanos = to_nanos(CompleteStr(&after[1..])).expect(after);
                    }
                }
                seconds += u64::from_str(body)? * *mul;
            } else {
                ensure!(body.is_empty(), "unexpected data preceding 'T': {:?}", body)
            }

            last = idx + 1;
        }
    }

    ensure!(
        parts.next().is_none() && last == input.len(),
        "unexpected trailing data: {:?}",
        &input[last..]
    );

    Ok(Duration::new(seconds, nanos))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

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

    #[test]
    fn duration() {
        use super::parse;
        assert_eq!(Duration::new(7, 0), parse("PT7S").unwrap());
        assert_eq!(Duration::new(7, 5_000_000), parse("PT7.005S").unwrap());
        assert_eq!(Duration::new(2 * 60, 0), parse("PT2M").unwrap());
        assert_eq!(
            Duration::new((2 * 24 + 1) * 60 * 60, 0),
            parse("P2DT1H").unwrap()
        );
    }

    #[test]
    fn parts() {
        let mut p = super::Parts::new("1D23M");
        assert_eq!(Some(("1", 'D')), p.next());
        assert_eq!(Some(("23", 'M')), p.next());
    }
}
