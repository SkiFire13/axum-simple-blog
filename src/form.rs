use std::path::Path;

use axum::{
    body::Bytes,
    extract::{Multipart, State},
    http::StatusCode,
    response::{Redirect, Result},
};
use chrono::Utc;
use reqwest::Client;
use uuid::Uuid;

use crate::AppState;

pub async fn form(State(state): State<AppState>, multipart: Multipart) -> Result<Redirect> {
    log::info!("Received /form request");

    let date = Utc::now();
    let uuid = Uuid::new_v4();
    let form = extract_form_field(multipart).await?;

    let image_filename = format!("image-{uuid}.png");
    let avatar_filename = format!("avatar-{uuid}.png");

    if let Some(image) = &form.image {
        save_image_file(image, &image_filename, &state.images_dir).await?;
    }

    if let Some(avatar_url) = &form.avatar_url {
        let avatar = retrieve_avatar(&avatar_url, &state.client)
            .await
            .inspect_err(|e| log::error!("failed to load avatar image: {e:?}"))
            .map_err(|_| (StatusCode::BAD_REQUEST, "Failed to load avatar image"))?;

        save_image_file(&avatar, &avatar_filename, &state.images_dir).await?;
    }

    let image_file = form.image.is_some().then(|| image_filename);
    let avatar_file = form.avatar_url.is_some().then(|| avatar_filename);

    sqlx::query!(
        "INSERT INTO blogposts (id, date, text, image, user, avatar) VALUES ($1, $2, $3, $4, $5, $6)",
        uuid, date, form.text, image_file, form.user, avatar_file
    )
    .execute(&state.db_pool)
    .await
    .inspect_err(|e| log::error!("failed to insert blogpost: {e}"))
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Redirect::to("/home"))
}

struct Form {
    text: String,
    image: Option<Bytes>,
    user: String,
    avatar_url: Option<String>,
}

async fn extract_form_field(mut multipart: Multipart) -> Result<Form> {
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

    let text = text
        .filter(|text| !text.is_empty())
        .ok_or(StatusCode::BAD_REQUEST)?;
    let image = image.filter(|image| !image.is_empty());
    let user = user
        .filter(|user| !user.is_empty())
        .ok_or(StatusCode::BAD_REQUEST)?;
    let avatar_url = avatar_url.filter(|avatar_url| !avatar_url.is_empty());

    Ok(Form {
        text,
        image,
        user,
        avatar_url,
    })
}

async fn save_image_file(image: &Bytes, image_filename: &str, image_dir: &Path) -> Result<()> {
    // This is assuming that a file with the same path doesn't exist yet.
    tokio::fs::write(image_dir.join(&image_filename), image)
        .await
        .inspect_err(|e| log::error!("failed to save image file: {e}"))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(())
}

async fn retrieve_avatar(url: &str, client: &Client) -> Result<Bytes, reqwest::Error> {
    // TODO: Put a limit on the size to avoid DOS attacks
    client.get(url).send().await?.bytes().await
}
