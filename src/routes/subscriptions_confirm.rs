use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String
}

#[tracing::instrument(
    name = "Confirm a pending subscriber",
    skip(_parameters),
)]
pub async fn confirm(
    _parameters: web::Query<Parameters>,
    db_pool: web::Data<PgPool>
) -> HttpResponse {
    let id = match get_subscriber_id_from_token(&db_pool, &_parameters.subscription_token).await {
        Ok(id) => id,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    match id {
        // If the subscription token does not exist
        None => HttpResponse::Unauthorized().finish(),
        Some(subscription_id) => {
            if confirm_subscriber(&db_pool, subscription_id).await.is_err() {
                return HttpResponse::InternalServerError().finish()
            }
            HttpResponse::Ok().finish()
        }
    }
}

#[tracing::instrument(
    name="Mark subscriber as confirmed in database",
    skip(db_pool, subscription_id),
)]
pub async fn confirm_subscriber(
    db_pool: &PgPool,
    subscription_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1"#,
        subscription_id
    )
        .execute(db_pool)
        .await
        .map_err(|e|{
            tracing::error!("Failed to execute `confirm_subscriber` query: {:?}", e);
            e
        })?;
    Ok(())
}

#[tracing::instrument(
    name="Get subscriber id from token",
    skip(db_pool)
)]
pub async fn get_subscriber_id_from_token(
    db_pool: &PgPool,
    subscription_token: &str
) -> Result<Option<Uuid>, sqlx::Error> {
    let subscription_id = sqlx::query!(
        r#"SELECT subscription_id from subscription_tokens WHERE subscription_token = $1"#,
        subscription_token
    )
        .fetch_optional(db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch `subscription_token` from `get_subscriber_id_from_token`: {:?}", e);
            e
        })?;

    // Optionally return if the subscription token is found in the database
    Ok(subscription_id.map(|r| r.subscription_id))
}