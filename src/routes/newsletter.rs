use std::fmt::{Debug, Display, Formatter};
use actix_web::{web, HttpResponse, ResponseError};
use actix_web::http::StatusCode;
use anyhow::Context;
use sqlx::PgPool;
use crate::domain::subscriber_email::SubscriberEmail;
use crate::email_client::EmailClient;
use crate::routes::error_chain_fmt;

#[derive(serde::Deserialize)]
pub struct BodyData {
    subject: String,
    text: String,
    category: String,
}

struct ConfirmedSubscriber {
    email: SubscriberEmail
}

#[derive(thiserror::Error)]
pub enum PublishError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error)
}

impl Debug for PublishError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for PublishError {
    fn status_code(&self) -> StatusCode {
        match self {
            PublishError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

// #[derive(serde::Deserialize)]
// pub struct Content {
//     text: String
// }

// Dummy implementation for newsletter endpoint
pub async fn publish_newsletter(
    body: web::Json<BodyData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>
) -> Result<HttpResponse, PublishError> {
    let subscribers = get_confirmed_subscribers(&pool).await?;

    for subscriber in subscribers {
        email_client
            .send_email(
                subscriber.email,
                &body.subject,
                &body.text,
                &body.category
            )
            .await
            // using `.with_context` instead of `.context` function
            .with_context(|| {
                // with_context us utilized due to the runtime cost of error handling
                // as the subscriber's email will not be static and will be only available at runtime
               format!("Failed to send newsletter email to subscriber: {}", subscriber.email)
            })?;

    }

    Ok(HttpResponse::Ok().finish())
}


#[tracing::instrument(
    name = "Get confirmed subscribers",
    skip(pool),
)]
async fn get_confirmed_subscribers(
    pool: &PgPool
) -> Result<Vec<ConfirmedSubscriber>, anyhow::Error>{

    // A temporary utility struct to convert the
    // String to a custom type `SubscriberEmail`
    struct Row {
        email: String
    }
    let rows = sqlx::query_as!(
        Row,
        r#"
            SELECT email
            FROM subscriptions
            WHERE status = 'confirmed'
        "#
    )
        .fetch_all(pool)
        .await?;

    let confirmed_subscribers = rows
        .into_iter()
        .map(|r| ConfirmedSubscriber {
            email: SubscriberEmail::parse(r.email).unwrap()
        })
        .collect();
    Ok(confirmed_subscribers)
}