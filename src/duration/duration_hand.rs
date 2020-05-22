use std::iter::Peekable;
use std::str::FromStr;
use std::time::Duration;

use anyhow::anyhow;
use anyhow::bail;
use anyhow::ensure;
use anyhow::Result;

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
                (init, point.as_bytes()[0].to_ascii_uppercase() as char)
            })
    }
}

fn maybe_take(parts: &mut Peekable<Parts>, token: char, mul: u64) -> Result<u64> {
    Ok(match parts.peek().cloned() {
        Some((body, found_token)) if found_token == token => {
            parts.next().unwrap();
            u64::from_str(body)? * mul
        }
        _ => 0,
    })
}

fn take_empty(parts: &mut Peekable<Parts>, token: char) -> Result<()> {
    match parts.next() {
        Some(("", avail)) if avail == token => Ok(()),
        Some((head, avail)) if avail == token => {
            bail!("invalid data before '{}': {:?}", token, head)
        }
        other => bail!("expected '{}', not {:?}", token, other),
    }
}

pub fn parse(input: &str) -> Result<Duration> {
    let mut parts = Parts::new(input).peekable();

    let mut seconds = 0u64;
    let mut nanos = 0u32;

    take_empty(&mut parts, 'P')?;

    seconds += maybe_take(&mut parts, 'W', super::SECS_PER_WEEK)?;
    seconds += maybe_take(&mut parts, 'D', super::SECS_PER_DAY)?;

    take_empty(&mut parts, 'T')?;

    seconds += maybe_take(&mut parts, 'H', super::SECS_PER_HOUR)?;
    seconds += maybe_take(&mut parts, 'M', super::SECS_PER_MINUTE)?;

    if let Some((mut body, 'S')) = parts.peek() {
        parts.next().unwrap();

        if let Some(first_point) = body.find('.') {
            let (main, after) = body.split_at(first_point);
            body = main;
            nanos = super::to_nanos(&after[1..]).ok_or(anyhow!("invalid nanos"))?;
        }

        seconds += u64::from_str(body)?;
    }

    ensure!(
        parts.peek().is_none(),
        "unexpected trailing data: {:?}",
        parts.next().unwrap(),
    );

    Ok(Duration::new(seconds, nanos))
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

    #[test]
    fn parts() {
        let mut p = super::Parts::new("1D23M");
        assert_eq!(Some(("1", 'D')), p.next());
        assert_eq!(Some(("23", 'M')), p.next());
    }
}
