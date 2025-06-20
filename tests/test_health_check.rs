use std::net::TcpListener;
use once_cell::sync::Lazy;
use secrecy::ExposeSecret;
use sqlx::{PgConnection, Connection, PgPool, Executor};
use uuid::Uuid;
use email_newsletter_rust::configuration::{get_configuration, DatabaseSettings, Settings};
use email_newsletter_rust::telemetry::{get_subscriber, init_subscriber};

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

#[tokio::test]
async fn test_health_check() {

    //No .await and no .expect required here
    let address = spawn_app().await.address;

    let client = reqwest::Client::new();
    let response = client.get(&format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn test_subscribe_returns_200_for_valid_data() {
    let test_config = spawn_app().await;

    // let configuration = get_configuration().expect("Failed to get Configuration!");
    let configuration = test_config.configuration;
    let db_conn = configuration.database.with_db();

    // The "Connection" trait must be in scope to invoke
    // `PgConnection::connect`
    // it is not an inherent method of the struct
    // hence we also need to import `Connection` trait from sqlx
    let mut connection = PgConnection::connect_with(&db_conn)
        .await
        .expect("Failed to connect to Postgres");

    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &test_config.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn test_subscribe_returns_400_when_fields_are_present_but_invalid() {
    let address = spawn_app().await.address;
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=le%20guin%40gmail.com", "missing email"),
        ("email=ursula_le_guin%40gmail.com", "missing name"),
        ("", "missing email and name both"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            400,
            response.status().as_u16(),
            "API did not fail with 400 error code: {}",
            error_message
        );
    }
}

#[test]
fn dummy_fail() {
    // the "claim" dependency can be utilized to get details regarding
    // In case if Err is returned while testing return type `Result<Ok,Err>` and when Err is returned
    // there is no justification regarding the error, leaving all the details hidden for that test case
    let result: Result<&str, &str> = Err("This is a dummy test");
    claim::assert_err!(result);
}

/// Spin up the application in the background
/// Return the address of the application i.e localhost:XXXX
async fn spawn_app() -> TestApp {

    // The first time `initialize` is invoked the code in `TRACING` is executed.
    // Next invocations get skipped
    Lazy::force(&TRACING);

    // We are not using the port 9001 here, instead we are binding to a random port provided by OS
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    // Create db connection using PgPool(Pool) implementation of sqlx
    // randomize the db mame and use it for testing
    let mut configuration = get_configuration().expect("Failed to get Configuration in spawn_app");
    configuration.database.database_name = Uuid::new_v4().to_string();

    let db_pool = configure_database(&configuration.database).await;

    // Here we dont .await the call, instead run the process in the background using tokio::spawn function
    // and return the server handle
    let server = email_newsletter_rust::startup::run(listener, db_pool.clone()).expect("Failed to bind address");
    let _ = tokio::spawn(server);

    TestApp {
        address,
        db_pool,
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