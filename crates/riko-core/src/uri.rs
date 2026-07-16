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

pub fn parse_join_uri(uri: &str) -> Option<u32> {
    let parsed = Url::parse(uri).ok()?;
    if parsed.scheme() != "riko" {
        return None;
    }
    let host_is_join = parsed.host_str() == Some("join");
    let path_is_join = parsed.path().trim_matches('/') == "join";
    if !host_is_join && !path_is_join {
        return None;
    }
    parsed
        .query_pairs()
        .find(|(k, _)| k == "game")
        .and_then(|(_, v)| v.parse::<u32>().ok())
}

pub fn join_link(game_id: u32) -> String {
    format!("riko://join?game={game_id}")
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

    #[test]
    fn parse_join_links() {
        assert_eq!(parse_join_uri("riko://join?game=42"), Some(42));
        assert_eq!(parse_join_uri(&join_link(7)), Some(7));
        assert!(parse_join_uri("riko://join").is_none());
        assert!(parse_join_uri("riko://play?game=1").is_none());
        assert!(parse_join_uri("vortex://join?game=1").is_none());
    }
}
