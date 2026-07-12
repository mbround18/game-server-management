use std::fmt::Error;

/// Parses common truthy/falsey string values into a boolean.
///
/// Accepted truthy values: `"true"`, `"1"`.
/// Accepted falsey values: `"false"`, `"0"`.
/// Any other value is treated as `false`.
///
/// # Errors
///
/// This function currently never returns `Err`; it always maps the input to `Ok(bool)`.
pub fn parse_truthy(value: &str) -> Result<bool, Error> {
    Ok(matches!(value.to_lowercase().as_str(), "true" | "1"))
}

// test the parse_truthy function
#[test]
fn test_parse_truthy() {
    assert_eq!(parse_truthy("true"), Ok(true));
    assert_eq!(parse_truthy("false"), Ok(false));
    assert_eq!(parse_truthy("1"), Ok(true));
    assert_eq!(parse_truthy("0"), Ok(false));
    assert_eq!(parse_truthy(""), Ok(false));
    assert_eq!(parse_truthy("qwdqwdqwd"), Ok(false));
}
