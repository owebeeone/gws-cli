#[cfg(test)]
use clap::CommandFactory;

pub(crate) fn parse_non_negative_i64(value: &str) -> Result<i64, String> {
    let parsed = value
        .parse::<i64>()
        .map_err(|_| "--progress-interval requires an integer".to_owned())?;
    if parsed < 0 {
        return Err("--progress-interval must be zero or greater".to_owned());
    }
    Ok(parsed)
}
