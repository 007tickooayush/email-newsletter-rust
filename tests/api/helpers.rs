use std::net::TcpListener;
use once_cell::sync::Lazy;
use uuid::Uuid;
use email_newsletter_rust::configuration::{get_configuration, DatabaseSettings, Settings};
use email_newsletter_rust::email_client::EmailClient;
use email_newsletter_rust::telemetry::{get_subscriber, init_subscriber};
use sqlx::{PgConnection, Connection, PgPool, Executor};
use email_newsletter_rust::startup::{get_connection_pool, Application};

// Ensure that the `tracing` stack is only initialized once rather than for each test case
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_lvl = "info".into();
    let subscriber_name = "test".into();

    // Cannot assign the output of `get_subscriber` to a variable based on the value of `TEST_LOG`
    // because the sink is part of the type returned by `get_subscriber`, therefore they are not the
    // same type. To work around it, but this is the most straight-forward way of moving forward.
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_lvl, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_lvl, std::io::sink);
        init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
    pub configuration: Settings
}


/// Spin up the application in the background
/// Return the address of the application i.e localhost:XXXX
pub async fn spawn_app() -> TestApp {

    // The first time `initialize` is invoked the code in `TRACING` is executed.
    // Next invocations get skipped
    Lazy::force(&TRACING);

    // // We are not using the port 9001 here, instead we are binding to a random port provided by OS
    // let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    // let port = listener.local_addr().unwrap().port();
    // let address = format!("http://127.0.0.1:{}", port);

    // Randomized the configuration to ensure test isolation
    let configuration = {
        // Create db connection using PgPool(Pool) implementation of sqlx
        // randomize the db mame and use it for testing
        let mut c = get_configuration().expect("Failed to get Configuration in spawn_app");
        c.database.database_name = Uuid::new_v4().to_string();
        c.application.port = 0;
        c
    };

    // let db_pool = configure_database(&configuration.database).await;

    // // Create a new `EmailClient` using the configuration
    // let sender_email = configuration.email_client.sender()
    //     .expect("Invalid Sender Email Address");
    //
    // let sender_name = configuration.email_client.sender_name()
    //     .expect("Invalid Sender Name");
    // let timeout = configuration.email_client.timeout();
    // let email_client = EmailClient::new(
    //     // using clone as the configuration object is partially moved
    //     configuration.email_client.base_url.clone(),
    //     sender_email,
    //     sender_name,
    //     configuration.email_client.authorization_token.clone(),
    //     timeout
    // );

    // Here we dont .await the call, instead run the process in the background using tokio::spawn function
    // and return the server handle

    // Launch the server using the configuration built
    let application = Application::build(configuration.clone()) // utilizing .clone() to avoid moving the configuration
        .await
        .expect("Failed to build server");

    let address = format!(
        "http://127.0.0.1:{}"
        ,application.port()
    );
    let _ = tokio::spawn(application.run_until_stopped());

    // Get the address of the server
    TestApp {
        address,
        db_pool: get_connection_pool(&configuration.database),
        configuration
    }
}


/// NOTE: there is no cleanup mechanism implemented the created databases using random V4 UUIDs
/// For handling the complete process properly the active Database connections need to be terminated,
/// And the databases need to be dropped
/// ---
/// That can be achieved using the "sqlx::test" macro
///
/// For more information regarding the issue check:
/// https://stackoverflow.com/questions/73013414/drop-database-on-drop-using-sqlx-and-rust
///
pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // create database
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to establish connection in configure_database");

    connection
        .execute(format!(r#"
            CREATE DATABASE "{}";
        "#, config.database_name).as_str())
        .await
        .expect("FAILED to CREATE DATABASE configure_database");

    // Database migrations
    let db_conn_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to bind address for db spawn_app");

    sqlx::migrate!("./migrations")
        .run(&db_conn_pool)
        .await
        .expect("Failed to exectute migration of database");

    db_conn_pool
}