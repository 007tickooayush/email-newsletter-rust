use std::net::TcpListener;
use env_logger::Env;
use sqlx::{Connection, PgPool};
use crate::configuration::get_configuration;
use crate::startup::run;

mod routes;
mod configuration;
mod startup;


#[tokio::main]
async fn main() -> std::io::Result<()> {

    // `init` calls set_logger so this is all we need to do
    // Ww are falling back to printing all logs at info level or above
    // in case the RUST_LOG environment variable is not set
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();


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
