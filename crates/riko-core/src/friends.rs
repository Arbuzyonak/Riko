use crate::{net, RikoError, VORTEX_BASE};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct Friend {
    pub id: u64,
    pub username: String,
    pub online_status: String,
    pub avatar: Option<String>,
}

pub async fn fetch_friends(token: &str) -> Result<Vec<Friend>, RikoError> {
    let cookie = format!("session_token={token}");
    let resp = net::send_retrying(
        || {
            net::shared()
                .get(format!("{VORTEX_BASE}/api/friends"))
                .header("Cookie", &cookie)
        },
        3,
    )
    .await?
    .error_for_status()?;
    let body: Vec<serde_json::Value> = resp.json().await?;
    let mut friends: Vec<Friend> = body.iter().filter_map(parse_friend).collect();

    if !friends.is_empty() {
        let ids = friends
            .iter()
            .map(|f| f.id.to_string())
            .collect::<Vec<_>>()
            .join(",");
        if let Ok(resp) = net::shared()
            .get(format!("{VORTEX_BASE}/api/users/avatar-pictures?ids={ids}"))
            .header("Cookie", &cookie)
            .send()
            .await
            && let Ok(avatars) = resp.json::<serde_json::Value>().await
        {
            for friend in &mut friends {
                friend.avatar = avatars
                    .get(friend.id.to_string())
                    .and_then(|v| v.as_str())
                    .filter(|s| s.starts_with("data:image/"))
                    .map(str::to_string);
            }
        }
    }

    friends.sort_by_key(|f| match f.online_status.as_str() {
        "in_game" => 0,
        "online" => 1,
        _ => 2,
    });
    Ok(friends)
}

fn parse_friend(body: &serde_json::Value) -> Option<Friend> {
    Some(Friend {
        id: body.get("id")?.as_u64()?,
        username: body
            .get("username")?
            .as_str()
            .filter(|s| !s.is_empty())?
            .to_string(),
        online_status: body
            .get("online_status")
            .and_then(|v| v.as_str())
            .unwrap_or("offline")
            .to_string(),
        avatar: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_live_friend_shape() {
        let body =
            serde_json::json!({"id": 27116, "online_status": "in_game", "username": "BlueSourPach"});
        let friend = parse_friend(&body).unwrap();
        assert_eq!(friend.id, 27116);
        assert_eq!(friend.username, "BlueSourPach");
        assert_eq!(friend.online_status, "in_game");
        assert!(parse_friend(&serde_json::json!({"id": 1})).is_none());
    }
}
