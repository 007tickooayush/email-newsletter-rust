use actix_web::{web, HttpResponse};
use sqlx::{PgPool};
use chrono::Utc;
use tracing::Instrument;
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
    let _req_span_guard = request_span.enter();

    // We do not call enter after the query_span
    let query_span = tracing::info_span!(
        "Saving new subscriber in database"
    );
    
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
        // attach `.instrument` and await it
        .instrument(query_span)
        .await {
        Ok(_) => {
            HttpResponse::Ok().finish()
        },
        Err(e) => {
            tracing::error!(" request_id: {} Failed to execute query :{}", request_id, e);
            HttpResponse::InternalServerError().finish()
        }
    }
}