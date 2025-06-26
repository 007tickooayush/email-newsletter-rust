use std::net::TcpListener;
use secrecy::ExposeSecret;
use sqlx::Connection;
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::layer::SubscriberExt;
use crate::configuration::get_configuration;
use crate::email_client::EmailClient;
use crate::startup::run;
use crate::telemetry::{get_subscriber, init_subscriber};

mod routes;
mod configuration;
mod startup;

mod telemetry;

mod domain;
mod email_client;
mod email_request;

#[tokio::main]
async fn main() -> std::io::Result<()> {

    // Initializing the subscriber
    let subscriber = get_subscriber("email_newsletter_rust".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // Panic if we can't read the configuration file
    let configuration = get_configuration().expect("Failed to read configuration");

    // Using Pool implementation in order to handle concurrency of database query executions
    // only try to establish a connection when the pool is used for the first time.
    let connection = PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());

    // A new `EmailClient` created using `configuration`
    let sender_email = configuration.email_client.sender()
        .expect("Invalid Sender Email Address");
    let sender_name = configuration.email_client.sender_name()
        .expect("Invalid Sender Name");
    let timeout = configuration.email_client.timeout();
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender_email,
        sender_name,
        configuration.email_client.authorization_token,
        timeout
    );

    // Remove the hardcoded 9001 port
    let address = format!("{}:{}", configuration.application.host , configuration.application.port);
    let listener = TcpListener::bind(address)?;

    // Bubble up the io::Error if we failed to bind the address
    // Or else just .await on Server
    run(listener, connection, email_client)?.await
}
