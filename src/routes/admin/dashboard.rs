use actix_web::{web, HttpResponse};
use actix_web::http::header::{ContentType, LOCATION};
use sqlx::PgPool;
use uuid::Uuid;
use anyhow::Context;
use crate::session_state::TypedSession;
// required for get_username anyhow::Error handling

pub async fn admin_dashboard(
    session: TypedSession,
    pool: web::Data<PgPool>
) -> Result<HttpResponse, actix_web::Error> {
    let username = if let Some(user_id) = session.get_user_id().map_err(e500)? {
        get_username(user_id, &pool).await.map_err(e500)?
    } else {
        return Ok(HttpResponse::SeeOther()
            .insert_header((LOCATION, "/login"))
            .finish());
    };
    Ok(
        HttpResponse::Ok()
            .content_type(ContentType::html())
            .body(format!(
                r#"
                <!DOCTYPE html>
                <html lang="en">
                <head>
                <meta http-equiv="content-type" content="text/html; charset=utf-8">
                <title>Admin dashboard</title>
                </head>
                <body>
                <p>Welcome {username}!</p>
                </body>
                </html>
                "#
            ))
    )
}

#[tracing::instrument(
    name = "Get username",
    skip(pool)
)]
async fn get_username(
    user_id: Uuid,
    pool: &PgPool
) -> Result<String, anyhow::Error> {
    let row = sqlx::query!(
        r#"
            SELECT username
            FROM users
            WHERE user_id = $1
        "#,
        user_id
    )
        .fetch_one(pool)
        .await
        .context("failed to fetch username")?;
    Ok(row.username)
}

/// Function to return an opaque 500 while logging the cause
fn e500<T>(e: T) -> actix_web::Error
where
    T: std::fmt::Debug + std::fmt::Display + 'static
{
    actix_web::error::ErrorInternalServerError(e)
}