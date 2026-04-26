use reqwest::Url;

pub fn is_valid_url(input: &str) -> bool {
    Url::parse(input).is_ok()
}

#[cfg(test)]
mod tests {
    use super::is_valid_url;

    #[test]
    fn accepts_standard_https_urls() {
        assert!(is_valid_url("https://example.com/webhook"));
    }

    #[test]
    fn accepts_localhost_urls_with_ports() {
        assert!(is_valid_url("http://localhost:8080/health"));
    }

    #[test]
    fn accepts_ipv6_urls() {
        assert!(is_valid_url("http://[2001:db8::1]:8080/status"));
    }

    #[test]
    fn rejects_urls_without_a_scheme() {
        assert!(!is_valid_url("example.com/webhook"));
    }

    #[test]
    fn rejects_invalid_urls() {
        assert!(!is_valid_url("http://[::1"));
    }
}
