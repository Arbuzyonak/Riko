use url::Url;

pub fn parse_vortex_uri(uri: &str) -> Option<(u32, String)> {
    let parsed = Url::parse(uri).ok()?;
    if parsed.scheme() != "vortex" {
        return None;
    }
    let mut game_id = None;
    let mut token = None;
    for (key, value) in parsed.query_pairs() {
        match key.as_ref() {
            "game" => game_id = value.parse::<u32>().ok(),
            "token" => token = Some(value.into_owned()),
            _ => {}
        }
    }
    Some((game_id?, token?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_uri() {
        let result = parse_vortex_uri("vortex://play?game=4&token=abc123");
        assert_eq!(result, Some((4, "abc123".to_string())));
    }

    #[test]
    fn parse_invalid_scheme() {
        assert!(parse_vortex_uri("http://example.com").is_none());
    }

    #[test]
    fn parse_missing_token() {
        assert!(parse_vortex_uri("vortex://play?game=4").is_none());
    }
}
