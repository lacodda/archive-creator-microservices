use bytesize::ByteSize;
use std::str::FromStr;

pub fn parse_size(size_str: &str) -> Result<usize, String> {
    let size = ByteSize::from_str(size_str).map_err(|e| e.to_string())?;
    Ok(size.as_u64() as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_size() {
        assert_eq!(parse_size("1kb").unwrap(), 1024);
        assert_eq!(parse_size("1mb").unwrap(), 1024 * 1024);
        assert_eq!(parse_size("1gb").unwrap(), 1024 * 1024 * 1024);
        assert!(parse_size("invalid").is_err());
    }
}
