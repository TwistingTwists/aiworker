pub mod jsonish;

#[cfg(test)]
mod tests {
    use super::jsonish;
    use jsonish::ParseOptions;

    #[test]
    fn test_valid_json_object() {
        let input = r#"{\"name\": \"Alice\", \"age\": 30}"#;
        assert!(jsonish::parse(input, ParseOptions::default()).is_ok());
    }

    #[test]
    fn test_invalid_json_missing_comma() {
        let input = r#"{\"name\": \"Bob\" \"age\": 25}"#;
        assert!(jsonish::parse(input, ParseOptions::default()).is_err());
    }

    #[test]
    fn test_nested_json_structures() {
        let input = r#"{\"users\": [{\"id\": 1}, {\"id\": 2}]}"#;
        assert!(jsonish::parse(input, ParseOptions::default()).is_ok());
    }

    #[test]
    fn test_unclosed_array() {
        let input = r#"[1, 2, 3"#;
        assert!(jsonish::parse(input, ParseOptions::default()).is_err());
    }
}
