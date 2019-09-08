use std::time::Duration;

use failure::Error;
use nom::IResult;
use nom::bytes::complete::tag;

fn num(input: &str) -> IResult<&str, u64> {
    let (input, num) = nom::character::complete::digit1(input)?;
    Ok((input, num.parse().expect("TODO")))
}

fn period_num(input: &str) -> IResult<&str, (u64, u32)> {
    let (input, whole) = num(input)?;
    if !input.starts_with(".") {
        return Ok((input, (whole, 0)));
    }

    let (input, _) = tag(".")(input)?;
    let (input, frac) = nom::character::complete::digit1(input)?;

    Ok((input, (whole, super::to_nanos(frac).expect("TODO"))))
}

fn time(input: &str) -> IResult<&str, (u64, u32)> {
    let (input, _) = tag("T")(input)?;
    let (input, h) = nom::combinator::opt(nom::sequence::terminated(num, tag("H")))(input)?;
    let (input, m) = nom::combinator::opt(nom::sequence::terminated(num, tag("M")))(input)?;
    let (input, s) = nom::combinator::opt(nom::sequence::terminated(period_num, tag("S")))(input)?;

    Ok((input,
        (
            (  h.unwrap_or(0) * super::SECS_PER_HOUR
                   + m.unwrap_or(0) * super::SECS_PER_MINUTE
                   + s.map(|(s, _ns)|  s).unwrap_or(0),
               s.map(|(_s, ns)| ns).unwrap_or(0)
            )
        )
    ))
}

fn period(input: &str) -> IResult<&str, (u64, u32)> {
    let (input, _) = tag("P")(input)?;
    let (input, w) = nom::combinator::opt(nom::sequence::terminated(num, tag("W")))(input)?;
    let (input, d) = nom::combinator::opt(nom::sequence::terminated(num, tag("D")))(input)?;
    let (input, t) = nom::combinator::opt(time)(input)?;

    Ok((input,
        (  w.unwrap_or(0) * super::SECS_PER_WEEK
               + d.unwrap_or(0) * super::SECS_PER_DAY
               + t.map(|(s, _ns)|  s).unwrap_or(0),
           t.map(|(_s, ns)| ns).unwrap_or(0)
        )
    ))
}

pub fn parse(input: &str) -> Result<Duration, Error> {
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
