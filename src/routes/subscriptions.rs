use actix_web::{web, HttpResponse};
use chrono::Utc;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sqlx::PgPool;
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
) -> HttpResponse {

    // This can also be written as `NewSubscriber::try_from(form.0)`
    // The try_into(TryInto) implementation is provided for free by the `TryFrom` trait
    let new_subscriber = match form.0.try_into() {
        Ok(subscriber) => subscriber,
        Err(_) => {
            // If the subscriber data is invalid, we return a BadRequest response
            return HttpResponse::BadRequest().finish();
        }
    };
    let subscription_id = match  insert_subscriber(&connection, &new_subscriber).await {
        Ok(subscriber_id) => subscriber_id,
        Err(_) => return HttpResponse::InternalServerError().finish()
    };

    // Get the new generated subscription token
    let subscription_token = generate_subscription_token();
    if store_token(&connection, subscription_id, &subscription_token)
        .await
        .is_err() {
        return HttpResponse::InternalServerError().finish();
    }

    if send_confirmation_email(
        &email_client,
        new_subscriber,
        &base_url.0,
        &subscription_token // dynamic token assignment
    )
        .await
        .is_err() {
        return  HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
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
    skip(new_subscriber, connection_pool),
)]
pub async fn insert_subscriber(
    connection_pool: &PgPool,
    new_subscriber: &crate::domain::new_subscriber::NewSubscriber,

) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();
    sqlx::query!(
        r#"
            INSERT INTO subscriptions (id, email, name, subscribed_at, status)
            VALUES ($1, $2, $3, $4, 'confirmed')
        "#,
        subscriber_id,
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
        .execute(connection_pool)
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
    skip()
)]
pub async fn store_token(
    db_pool: &PgPool,
    subscriber_id: Uuid,
    subscription_token: &str,
) -> Result<(), sqlx::Error> {
    let result = sqlx::query!(r#"
    INSERT INTO subscription_tokens (subscription_token, subscription_id) VALUES ($1, $2)
    "#,
        subscription_token,
        subscriber_id
    )
        .fetch_optional(db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute `storage_token` query: {:?}", e);
           e
        })?;

    Ok(())
}