#[cfg(test)]
use clap::CommandFactory;

pub(crate) fn parse_positive_i64(value: &str) -> Result<i64, String> {
    let parsed = value
        .parse::<i64>()
        .map_err(|_| "--jobs requires an integer".to_owned())?;
    if parsed < 1 {
        return Err("--jobs must be greater than zero".to_owned());
    }
    Ok(parsed)
}
