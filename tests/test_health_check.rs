use std::net::TcpListener;
use sqlx::{PgConnection, Connection, PgPool};
use email_newsletter_rust::configuration::get_configuration;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool
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
    let address = spawn_app().await.address;

    let configuration = get_configuration().expect("Failed to get Configuration!");
    let db_conn_string = configuration.database.connection_string();

    // The "Connection" trait must be in scope to invoke
    // `PgConnection::connect`
    // it is not an inherent method of the struct
    // hence we also need to import `Connection` trait from sqlx
    let mut connection = PgConnection::connect(&db_conn_string)
        .await
        .expect("Failed to connect to Postgres");

    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &address))
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
async fn test_subscribe_returns_400_for_missing_data() {
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

/// Spin up the application in the background
/// Return the address of the application i.e localhost:XXXX
async fn spawn_app() -> TestApp {
    // We are not using the port 9001 here, instead we are binding to a random port provided by OS
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    // Create db connection using PgPool(Pool) implementation of sqlx
    let configuration = get_configuration().expect("Failed to get Configuration in spawn_app");
    let db_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to bind address for db spawn_app");

    // Here we dont .await the call, instead run the process in the background using tokio::spawn function
    // and return the server handle
    let server = email_newsletter_rust::startup::run(listener, db_pool.clone()).expect("Failed to bind address");
    let _ = tokio::spawn(server);

    TestApp {
        address,
        db_pool
    }
}