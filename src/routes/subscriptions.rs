use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;
use crate::domain::new_subscriber::NewSubscriber;
use crate::domain::subscriber_email::SubscriberEmail;
use crate::domain::subscriber_name::SubscriberName;
use crate::email_client::EmailClient;

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
    skip(form, connection, email_client),
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
    email_client: web::Data<EmailClient>
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
    if insert_subscriber(&connection, &new_subscriber).await.is_err() {
        return HttpResponse::InternalServerError().finish();
    }

    // Added a static confirmation link
    let confirmation_link = "https://my-api.com/subscriptions/confirm";
    // Send a static email to the new subscriber
    if email_client.send_email(
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
        .is_err() {
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
}

#[tracing::instrument(
    name = "Inserting new subscriber",
    skip(new_subscriber, connection_pool),
)]
pub async fn insert_subscriber(
    connection_pool: &PgPool,
    new_subscriber: &crate::domain::new_subscriber::NewSubscriber,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
            INSERT INTO subscriptions (id, email, name, subscribed_at, status)
            VALUES ($1, $2, $3, $4, 'confirmed')
        "#,
        Uuid::new_v4(),
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
    Ok(())
}