use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::{Redirect, Result},
};

use crate::AppState;

pub async fn form(State(state): State<AppState>, mut multipart: Multipart) -> Result<Redirect> {
    let mut text = None;
    let mut image = None;
    let mut user = None;
    let mut avatar = None;

    while let Some(field) = multipart.next_field().await? {
        match field.name() {
            Some("text") if text.is_none() => text = Some(field.text().await?),
            Some("image") if image.is_none() => image = Some(field.bytes().await?),
            Some("user") if user.is_none() => user = Some(field.text().await?),
            Some("avatar") if avatar.is_none() => avatar = Some(field.text().await?),
            _ => return Err(StatusCode::BAD_REQUEST.into()),
        }
    }

    let text = text.ok_or(StatusCode::BAD_REQUEST)?;
    let user = user.ok_or(StatusCode::BAD_REQUEST)?;

    dbg!(&text, &image, &user, &avatar);

    Ok(Redirect::to("/home"))
}
