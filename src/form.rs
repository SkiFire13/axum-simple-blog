use axum::{
    body::Bytes,
    extract::{Multipart, State},
    http::StatusCode,
    response::{Redirect, Result},
};
use reqwest::Client;

use crate::AppState;

pub async fn form(State(state): State<AppState>, mut multipart: Multipart) -> Result<Redirect> {
    let mut text = None;
    let mut image = None;
    let mut user = None;
    let mut avatar_url = None;

    while let Some(field) = multipart.next_field().await? {
        match field.name() {
            Some("text") if text.is_none() => text = Some(field.text().await?),
            Some("image") if image.is_none() => image = Some(field.bytes().await?),
            Some("user") if user.is_none() => user = Some(field.text().await?),
            Some("avatar") if avatar_url.is_none() => avatar_url = Some(field.text().await?),
            _ => return Err(StatusCode::BAD_REQUEST.into()),
        }
    }

    let text = text.ok_or(StatusCode::BAD_REQUEST)?;
    let user = user.ok_or(StatusCode::BAD_REQUEST)?;

    let avatar = match avatar_url {
        Some(avatar) if avatar != "" => Some(
            retrieve_avatar(&avatar, &state.client)
                .await
                .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?,
        ),
        _ => None,
    };

    dbg!(&text, &image, &user, &avatar);

    Ok(Redirect::to("/home"))
}

async fn retrieve_avatar(url: &str, client: &Client) -> Result<Bytes, reqwest::Error> {
    // TODO: Put a limit on the size to avoid DOS attacks
    client.get(url).send().await?.bytes().await
}
