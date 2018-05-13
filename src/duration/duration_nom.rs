use std::time::Duration;

use failure::Error;
use nom::digit1;
use nom::types::CompleteStr;

named!(period_num<CompleteStr, (u64, u32)>,
    do_parse!(
        whole: num >>
        rest: opt!(preceded!(tag!("."), map_res!(digit1, super::to_nanos))) >>
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
          (  h.unwrap_or(0) * super::SECS_PER_HOUR
           + m.unwrap_or(0) * super::SECS_PER_MINUTE
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
          (  w.unwrap_or(0) * super::SECS_PER_WEEK
           + d.unwrap_or(0) * super::SECS_PER_DAY
           + t.map(|(s, _ns)|  s).unwrap_or(0),
             t.map(|(_s, ns)| ns).unwrap_or(0)
          )
        )
    ));

pub fn parse(input: &str) -> Result<Duration, Error> {
    match period(CompleteStr(input)) {
        Ok((CompleteStr(""), (s, ns))) => Ok(Duration::new(s, ns)),
        other => bail!("invalid: {:?}", other),
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

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
}
