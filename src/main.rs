use std::net::TcpListener;
use env_logger::Env;
use sqlx::{Connection, PgPool};
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{EnvFilter, Registry};
use tracing_subscriber::layer::SubscriberExt;
use crate::configuration::get_configuration;
use crate::startup::run;

mod routes;
mod configuration;
mod startup;


#[tokio::main]
async fn main() -> std::io::Result<()> {

    // REMOVED the env_logger::init() from the main function

    // Printing all spans at info-level
    // If the RUST_LOG env variable has not been set
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::from("info"));;

    let formatting_layer = BunyanFormattingLayer::new(
        "email_newsletter_rust".into(),
        // Output the logs into the stdout
        std::io::stdout
    );

    // the `with` function is provided by `SubscriberExt`, an extension trait
    // for `Subscriber` exposed by `tracing_subscriber`
    let subscriber = Registry::default()
        // with from `layer::SubscriberExt` trait
        .with(env_filter)
        // implementing JSON based logging foe Elasitcsearch-friendly architecture
        .with(JsonStorageLayer)
        .with(formatting_layer);

    // `tracing::subscriber::set_global_default` is utilized to specify the subscriber for span processing
    set_global_default(subscriber).expect("Failed to set Global Subscriber");

    // Panic if we can't read the configuration file
    let configuration = get_configuration().expect("Failed to read configuration");

    // USing Pool implementation in order to handle concurrency of database query executions
    let connection = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres [main]");

    // Remove the hardcoded 9001 port
    let address = format!("0.0.0.0:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;

    // Bubble up the io::Error if we failed to bind the address
    // Or else just .await on Server
    run(listener, connection)?.await
}
