use std::fmt::{Debug, Display, Formatter};
use actix_web::{web, HttpRequest, HttpResponse, ResponseError};
use actix_web::http::header::{HeaderMap, HeaderValue};
use actix_web::http::{header, StatusCode};
use anyhow::Context;
use argon2::{Algorithm, Argon2, Params, PasswordHash, PasswordVerifier, Version};
use secrecy::{ExposeSecret, Secret};
use sqlx::PgPool;
use crate::authentication::{validate_credentials, AuthError, Credentials};
use crate::domain::subscriber_email::SubscriberEmail;
use crate::email_client::EmailClient;
use crate::routes::error_chain_fmt;
use crate::telemetry::spawn_blocking_with_tracing;

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
    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error)
}

impl Debug for PublishError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for PublishError {
    fn error_response(&self) -> HttpResponse {
        match self {
            PublishError::UnexpectedError(_) => {
                HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
            },
            // Return a 401 status for Auth related Error
            PublishError::AuthError(_) => {
                let mut response = HttpResponse::new(StatusCode::UNAUTHORIZED);
                let header_value = HeaderValue::from_str(r#"Basic realm="publish""#)
                    .unwrap();
                response
                    .headers_mut()
                    // actix_web::http::header provides a collection of well-known/standard constants
                    // for HTTP requests
                    .insert(header::WWW_AUTHENTICATE, header_value);
                response
            },
        }
    }
}


#[tracing::instrument(
    name = "Publish a newsletter issue",
    skip(body, pool, email_client, request),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn publish_newsletter(
    body: web::Json<BodyData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    // added new extractor HttpRequest
    request: HttpRequest,
) -> Result<HttpResponse, PublishError> {
    let credentials = basic_authentication(request.headers())
        .map_err(PublishError::AuthError)?;
    tracing::Span::current().record(
        "username",
        &tracing::field::display(&credentials.username)
    );
    let user_id = validate_credentials(credentials, &pool)
        .await
        .map_err(|e| match e {
            AuthError::InvalidCredentials(_) => PublishError::AuthError(e.into()),
            AuthError::UnexpectedError(_) => PublishError::UnexpectedError(e.into())
        })?;
    tracing::Span::current().record(
        "user_id",
        &tracing::field::display(&user_id)
    );
    let subscribers = get_confirmed_subscribers(&pool).await?;

    for subscriber in subscribers {
        match subscriber {
            Ok(subscriber) => {
                email_client
                    .send_email(
                        &subscriber.email,
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
            },
            Err(error) => {
                tracing::warn!(
                    // Record the error chain as structured field
                    // on log record
                    error.cause_chain = ?error,
                    //
                    "Skipping a confirmed subscriber. \
                    Their stored contact details are invalid",
                );
            }
        }

    }

    Ok(HttpResponse::Ok().finish())
}


#[tracing::instrument(
    name = "Get confirmed subscribers",
    skip(pool),
)]
async fn get_confirmed_subscribers(
    pool: &PgPool
) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error>{

    // switched back to query!() from query_as!()
    let confirmed_subscribers = sqlx::query!(
        r#"
            SELECT email
            FROM subscriptions
            WHERE status = 'confirmed'
        "#
    )
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|r| match SubscriberEmail::parse(r.email) {
            Ok(email) => Ok(ConfirmedSubscriber { email }),
            Err(err) => Err(anyhow::anyhow!(err))
        })
        .collect();

    Ok(confirmed_subscribers)
}

fn basic_authentication(headers: &HeaderMap) -> Result<Credentials, anyhow::Error> {
    let header_value = headers
        .get("Authorization")
        .context("The 'Authorization' header was not found")?
        .to_str()
        .context("The 'Authorization' header was not a valid UTF8 string")?;

    let base64_encoded_segment = header_value
        .strip_prefix("Basic ")
        .context("The authorization scheme was not 'Basic'")?;
    let decoded_bytes = base64::decode_config(base64_encoded_segment, base64::STANDARD)
        .context("Failed to decode base64 'Basic' credentials")?;
    let decoded_credentials = String::from_utf8(decoded_bytes)
        .context("The decoded credential string is not valid UTF8")?;

    // Split the decoded credentials into two segments
    let mut credentials = decoded_credentials.splitn(2, ':');
    let username = credentials
        .next()
        .ok_or_else(|| anyhow::anyhow!("A username must be provided in 'Basic' auth."))?
        .to_string();
    let password = credentials
        .next()
        .ok_or_else(|| anyhow::anyhow!("A password must be provided in 'Basic' auth."))?
        .to_string();

    Ok(Credentials {
        username,
        password: Secret::new(password)
    })
}