use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;
use crate::domain::new_subscriber::NewSubscriber;
use crate::domain::subscriber_email::SubscriberEmail;
use crate::domain::subscriber_name::SubscriberName;

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

pub fn parse_subscriber(form: FormData) -> Result<NewSubscriber, String> {
    let name = SubscriberName::parse(form.name)?;
    let email = SubscriberEmail::parse(form.email)?;
    Ok(NewSubscriber {
        email,
        name
    })
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, connection),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    // Retrieving a connection from the application state!
    connection: web::Data<PgPool>,
) -> HttpResponse {

    let new_subscriber = match parse_subscriber(form.0) {
        Ok(subscriber) => subscriber,
        Err(_) => {
            // If the subscriber data is invalid, we return a BadRequest response
            return HttpResponse::BadRequest().finish();
        }
    };
    match insert_subscriber(&connection, &new_subscriber)
        .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish()
    }
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
            INSERT INTO subscriptions (id, email, name, subscribed_at)
            VALUES ($1, $2, $3, $4)
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