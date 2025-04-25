use std::net::TcpListener;
use sqlx::{PgConnection, Connection};
use crate::configuration::get_configuration;
use crate::startup::run;

mod routes;
mod configuration;
mod startup;


#[tokio::main]
async fn main() -> std::io::Result<()> {

    // Panic if we can't read the configuration file
    let configuration = get_configuration().expect("Failed to read configuration");

    let connection = PgConnection::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres [main]");

    // Remove the hardcoded 9001 port
    let address = format!("0.0.0.0:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;

    // Bubble up the io::Error if we failed to bind the address
    // Or else just .await on Server
    run(listener, connection)?.await
}
