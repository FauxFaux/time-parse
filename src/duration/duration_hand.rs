use std::iter::Peekable;
use std::str::FromStr;
use std::time::Duration;

use failure::Error;

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
        ("W", false, Some(super::SECS_PER_WEEK)),
        ("D", false, Some(super::SECS_PER_DAY)),
        ("T", false, None),
        ("H", false, Some(super::SECS_PER_HOUR)),
        ("M", false, Some(super::SECS_PER_MINUTE)),
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
                        nanos = super::to_nanos(&after[1..]).expect(after);
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
