use std::net::TcpListener;
use secrecy::ExposeSecret;
use sqlx::{Connection, PgPool};
use tracing_subscriber::layer::SubscriberExt;
use crate::configuration::get_configuration;
use crate::startup::run;
use crate::telemetry::{get_subscriber, init_subscriber};

mod routes;
mod configuration;
mod startup;

mod telemetry;

#[tokio::main]
async fn main() -> std::io::Result<()> {

    // Initializing the subscriber
    let subscriber = get_subscriber("email_newsletter_rust".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // Panic if we can't read the configuration file
    let configuration = get_configuration().expect("Failed to read configuration");

    // Using Pool implementation in order to handle concurrency of database query executions
    // only try to establish a connection when the pool is used for the first time.
    let connection = PgPool::connect_lazy(&configuration.database.connection_string().expose_secret())
        .expect("Failed to connect to Postgres [main]");

    // Remove the hardcoded 9001 port
    let address = format!("{}:{}", configuration.application.host , configuration.application.port);
    let listener = TcpListener::bind(address)?;

    // Bubble up the io::Error if we failed to bind the address
    // Or else just .await on Server
    run(listener, connection)?.await
}
