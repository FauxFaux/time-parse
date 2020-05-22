use std::time::Duration;

use anyhow::bail;
use anyhow::Result;
use nom::bytes::complete::tag;
use nom::combinator::opt;
use nom::sequence::terminated;
use nom::IResult;

fn num(input: &str) -> IResult<&str, u64> {
    let (input, num) = nom::character::complete::digit1(input)?;
    let num = num
        .parse()
        .map_err(|_| nom::Err::Error((input, nom::error::ErrorKind::TooLarge)))?;

    Ok((input, num))
}

fn period_num(input: &str) -> IResult<&str, (u64, u32)> {
    let (input, whole) = num(input)?;
    let (input, dot) = opt(tag("."))(input)?;
    if dot.is_none() {
        return Ok((input, (whole, 0)));
    }

    let (input, frac) = nom::character::complete::digit1(input)?;

    Ok((input, (whole, super::to_nanos(frac).expect("TODO"))))
}

fn time(input: &str) -> IResult<&str, (u64, u32)> {
    let (input, _) = tag("T")(input)?;
    let (input, h) = opt(terminated(num, tag("H")))(input)?;
    let (input, m) = opt(terminated(num, tag("M")))(input)?;
    let (input, s) = opt(terminated(period_num, tag("S")))(input)?;

    Ok((
        input,
        ((
            h.unwrap_or(0) * super::SECS_PER_HOUR
                + m.unwrap_or(0) * super::SECS_PER_MINUTE
                + s.map(|(s, _ns)| s).unwrap_or(0),
            s.map(|(_s, ns)| ns).unwrap_or(0),
        )),
    ))
}

fn period(input: &str) -> IResult<&str, (u64, u32)> {
    let (input, _) = tag("P")(input)?;
    let (input, w) = opt(terminated(num, tag("W")))(input)?;
    let (input, d) = opt(terminated(num, tag("D")))(input)?;
    let (input, t) = opt(time)(input)?;

    Ok((
        input,
        (
            w.unwrap_or(0) * super::SECS_PER_WEEK
                + d.unwrap_or(0) * super::SECS_PER_DAY
                + t.map(|(s, _ns)| s).unwrap_or(0),
            t.map(|(_s, ns)| ns).unwrap_or(0),
        ),
    ))
}

pub fn parse(input: &str) -> Result<Duration> {
    match period(input) {
        Ok(("", (s, ns))) => Ok(Duration::new(s, ns)),
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
        assert!(parse("PT2").is_err());
        assert!(parse("PT22").is_err());
        assert!(parse("PT2M2").is_err());
        assert!(parse("T2S").is_err());
        assert!(parse("P2S").is_err());
    }
}
