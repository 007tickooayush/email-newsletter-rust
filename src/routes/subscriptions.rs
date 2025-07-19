use std::error::Error;
use actix_web::{web, HttpResponse, ResponseError};
use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use chrono::Utc;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;
use crate::domain::new_subscriber::NewSubscriber;
use crate::domain::subscriber_email::SubscriberEmail;
use crate::domain::subscriber_name::SubscriberName;
use crate::email_client::EmailClient;
use crate::startup::ApplicationBaseUrl;

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

// TryFrom does not need t o be imported explicitly, as it is in the prelude
impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        Ok(Self { name,
            email
        })
    }
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, connection, email_client, base_url),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    // Retrieving a connection from the application state!
    connection: web::Data<PgPool>,
    // getting email_client instance from the application state
    email_client: web::Data<EmailClient>,
    // application server base url
    base_url: web::Data<ApplicationBaseUrl>
) -> Result<HttpResponse, SubscribeError> {

    let mut transaction = connection
        .begin()
        .await
            .map_err(|e| SubscribeError::UnexpectedError(Box::new(e)))?;

    // This can also be written as `NewSubscriber::try_from(form.0)`
    // The try_into(TryInto) implementation is provided for free by the `TryFrom` trait
    let new_subscriber = form
        .0
        .try_into()
        .map_err(|e| SubscribeError::UnexpectedError(Box::new(e)))?;

    let subscription_id = insert_subscriber(
        &mut transaction,
        &new_subscriber
    ).await
        .map_err(|e| SubscribeError::UnexpectedError(Box::new(e)))?;

    // Get the new generated subscription token
    let subscription_token = generate_subscription_token();
    store_token(
        &mut transaction,
        subscription_id,
        &subscription_token
    ).await
        .map_err(|e| SubscribeError::UnexpectedError(Box::new(e)))?;

    send_confirmation_email(
        &email_client,
        new_subscriber,
        &base_url.0,
        &subscription_token // dynamic token assignment
    ).await
        .map_err(|e| SubscribeError::UnexpectedError(Box::new(e)))?;;

    transaction
        .commit()
        .await
        .map_err(|e| SubscribeError::UnexpectedError(Box::new(e)))?;

    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(
    name = "Send a confirmation email to a new subscriber",
    skip(email_client, new_subscriber, base_url, subscription_token)
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
    base_url: &str,
    subscription_token: &str
) -> Result<(), reqwest::Error> {
    // Added a static confirmation link
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        base_url,
        subscription_token
    );
    // Send a static email to the new subscriber
    email_client
        .send_email(
            new_subscriber.email,
            "Weclome!",
            &format!(
                "Welcome to our newsletter! <br/> \
                Click <a href = \"{}\">here</a> to confirm your subscription",
                confirmation_link
            ),
            "welcome mail"
        )
        .await
}

#[tracing::instrument(
    name = "Inserting new subscriber",
    skip(new_subscriber, transaction),
)]
pub async fn insert_subscriber(
    transaction: &mut Transaction<'_, Postgres>,
    new_subscriber: &crate::domain::new_subscriber::NewSubscriber,

) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();
    sqlx::query!(
        r#"
            INSERT INTO subscriptions (id, email, name, subscribed_at, status)
            VALUES ($1, $2, $3, $4, 'pending_confirmation')
        "#, // default status is kept as pending_confirmation
        subscriber_id,
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
        .execute(transaction)
        .await
        .map_err(|e| {
            tracing::error!(" sqlx::Error::QueryBuilderError : {:?}", e);
            e
            // Using the ? operator to return early and propagate the error
            // return sqlx::error
        })?;
    Ok(subscriber_id)
}

/// using `rand` package's `std_rng` feature to generate a
/// "CryptoGraphically Secure Pseudo Number Generator" to generate subscription tokens
///
/// Generate a 25-character-long case-sensitive subscription token
fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

#[tracing::instrument(
    name = "Store the generated token in the database",
    skip(transaction, subscription_token)
)]
pub async fn store_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: Uuid,
    subscription_token: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(r#"
    INSERT INTO subscription_tokens (subscription_token, subscription_id) VALUES ($1, $2)
    "#,
        subscription_token,
        subscriber_id
    )
        .execute(transaction)
        .await.unwrap();
        // .map_err(|e| {
        //     tracing::error!("Failed to execute `storage_token` query: {:?}", e);
        //     e
        // })?;

    Ok(())
}

/// A new error type for wrapping `sqlx::Error`
/// because due to the "Oprhan Rule" in Rust we can not directly implement the `ResponseError`
/// for `sqlx::Error`.
///
/// ResponseError is utilixrd to provide better information regarding the errors propogating from
/// the database queries to utility functions and ending at API endpoint handlers
// #[derive(Debug)] // removed the default Debug implementation provided by rust
pub struct StoreTokenError(sqlx::Error);

impl ResponseError for StoreTokenError {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::InternalServerError().body(self.to_string())
    }
}

impl std::fmt::Debug for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\nCaused By:\n\t{}", self, self.0)
    }
}

impl std::fmt::Display for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "A database error occurred while trying to store a subscription token"
        )
    }
}

impl std::error::Error for StoreTokenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}


fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>
) -> std::fmt::Result {
    writeln!(f,"{}\n",e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}


/// new error type for handling errors with the understanding of HTTP protocol
/// implementing the error handling using `thiserror` package
///
/// `thiserror` removes the requirement of boilerplate code for error handling
#[derive(thiserror::Error)]
pub enum SubscribeError {
    #[error("{0}")]
    ValidationError(String),
    // Transparent delegates both `Display`'s and `source`'s implementation
    // to the type wrapped by `UnexpectedError`.
    #[error(transparent)]
    UnexpectedError(#[from] Box<dyn std::error::Error>)
}
impl std::fmt::Debug for SubscribeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self,f)
    }
}

impl ResponseError for SubscribeError {
    fn status_code(&self) -> StatusCode {
        match self {
            SubscribeError::ValidationError(_) => StatusCode::BAD_REQUEST,
            SubscribeError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}