use actix_web::{web, HttpResponse};
use sqlx::{PgPool};
use chrono::Utc;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String
}

pub async fn subscribe(
    form: web::Form<FormData>,
    // Retrieving a connection from the application state!
    connection: web::Data<PgPool>,
) -> HttpResponse {
    // Append a request ID to each set of execution in order to track the request in a better way
    let request_id = Uuid::new_v4();

    // Spans, like logs, have an associated level
    // 'info_Span' creates a span at "info-level"
    let request_span = tracing::info_span!(
        "Adding a new subscriber",
        %request_id,
        subscriber_email = %form.email,
        subscriber_name = %form.name
    );

    // Using `enter` in an async function is not advised
    // Check the section on `"Instrumenting Futures"`
    let _req = request_span.enter();


    tracing::info!(
      "request_id: {} - Adding '{}' '{}' as a subscriber",
        request_id,
        form.email,
        form.name
    );

    tracing::info!(" request_id: {} Saving new subscriber details in the database", request_id);
    match sqlx::query!(
        r#"
            INSERT INTO subscriptions (id, email, name, subscribed_at)
            VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
        .execute(connection.get_ref())
        .await {
        Ok(_) => {
            tracing::info!("request_id: {} New subscriber details have been saved", request_id);
            HttpResponse::Ok().finish()
        },
        Err(e) => {
            tracing::error!(" request_id: {} Failed to execute query :{}", request_id, e);
            HttpResponse::InternalServerError().finish()
        }
    }
}