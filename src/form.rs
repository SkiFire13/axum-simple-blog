use axum::{
    body::Bytes,
    extract::{Multipart, State},
    http::StatusCode,
    response::{Redirect, Result},
};
use reqwest::Client;
use uuid::Uuid;

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

    let uuid = Uuid::new_v4();

    let text = text.ok_or(StatusCode::BAD_REQUEST)?;
    let user = user.ok_or(StatusCode::BAD_REQUEST)?;

    let image_filename = format!("image-{uuid}.png");
    let avatar_filename = format!("avatar-{uuid}.png");

    if let Some(image) = &image {
        // This is assuming that a file with the same path doesn't exist yet.
        let path = state.image_dir.join(&image_filename);
        tokio::fs::write(path, image)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    let avatar_url = avatar_url.filter(|avatar_url| !avatar_url.is_empty());
    if let Some(avatar_url) = &avatar_url {
        // TODO: Is BAD_REQUEST here correct?
        let avatar = retrieve_avatar(&avatar_url, &state.client)
            .await
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

        // This is assuming that a file with the same path doesn't exist yet.
        let path = state.image_dir.join(&avatar_filename);
        tokio::fs::write(path, avatar)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    Ok(Redirect::to("/home"))
}

async fn retrieve_avatar(url: &str, client: &Client) -> Result<Bytes, reqwest::Error> {
    // TODO: Put a limit on the size to avoid DOS attacks
    client.get(url).send().await?.bytes().await
}
