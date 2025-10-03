use std::net::TcpListener;
use secrecy::ExposeSecret;
use sqlx::Connection;
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::layer::SubscriberExt;
use crate::configuration::get_configuration;
use crate::email_client::EmailClient;
use crate::startup::{run, Application};
use crate::telemetry::{get_subscriber, init_subscriber};

mod routes;
mod configuration;
mod startup;

mod telemetry;

mod domain;
mod email_client;
mod email_request;
mod authentication;

mod session_state;

mod utils;

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    // Initializing the subscriber
    let subscriber = get_subscriber("email_newsletter_rust".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // Panic if we can't read the configuration file
    let configuration = get_configuration().expect("Failed to read configuration");
    
    // Removed the boilerplate code for the `spawn_app` function
    let application = Application::build(configuration).await?;
    application.run_until_stopped().await?;    
    Ok(())
}
